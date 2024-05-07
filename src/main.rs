#![allow(dead_code)]
#![allow(unused_imports)]
// extern crate json_value_merge;

// use json_value_merge::Merge;
use jsonpath_lib::*;
use serde_json::{json, Map, Value};
use std::error::Error;
use std::str::FromStr;
extern crate jsonpath_lib as jsonpath;
use jsonpath::{replace_with, Selector};
use jsonpath_rust::{JsonPathFinder, JsonPathInst};
use regex::Regex;
use std::collections::HashMap;

use std::fs::File;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct RedactedObject {
    pub name: Value,
    pub path_index_count: i32,
    pub pre_path: Option<String>,
    pub post_path: Option<String>,
    pub original_path: Option<String>,
    pub final_path: Vec<Option<String>>,
    pub do_final_path_subsitution: bool,
    pub path_lang: Value,
    pub replacement_path: Option<String>,
    pub method: Value,
    pub reason: Value,
    pub result_type: Vec<Option<ResultType>>,
    pub redaction_type: Option<RedactionType>,
}

// These are the different types of results that we can get from the JSON path checks
// This is mainly used for debugging and attempting to figure what other strange weirdness we might hit
#[derive(Debug, PartialEq, Clone)]
pub enum ResultType {
    StringNoValue, // (*) what we found in the value paths array was a string but has no value (yes, this is a little weird, but does exist) `Redaction by Empty Value`
    EmptyString, // (*) what we found in the value paths array was a string but it is an empty string `Redaction by Empty Value`
    PartialString, // (*) what we found in the value paths array was a string and it does have a value `Redaction by Partial Value` and/or `Redaction by Replacement Value`
    Array, // what we found in the value paths array was _another_ array (have never found this)
    Object, // what we found in the value paths array was an object (have never found this)
    Removed, // (*) paths array is empty, finder.find_as_path() found nothing `Redaction by Removal`
    FoundNull, // value in paths array is null (have never found this)
    FoundNothing, // fall through, value in paths array is not anything else (have never found this)
    FoundUnknown, // what we found was not a JSON::Value::string (have never found this)
    FoundPathReturnedBadValue, // what finder.find_as_path() returned was not a Value::Array (have never found this, could possibly be an error)
}

// This isn't just based on the string type that is in the redaction method, but also based on the result type above
#[derive(Debug, PartialEq, Clone)]
pub enum RedactionType {
    EmptyValue,
    PartialValue,
    ReplacementValue,
    Removal,
    Unknown,
}

fn parse_redacted_array(v: &Value, redacted_array: &Vec<Value>) -> Vec<RedactedObject> {
    let mut result: Vec<RedactedObject> = Vec::new();

    for item in redacted_array {
        let item_map = item.as_object().unwrap();
        let pre_path = item_map
            .get("prePath")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let post_path = item_map
            .get("postPath")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        // we need set a final_path here, prefer pre over the post
        let original_path = pre_path.clone().or(post_path.clone());
        // let final_path: Vec<Option<String>> = Vec::new();
        let mut redacted_object = RedactedObject {
            name: Value::String(String::from("")), // Set to empty string initially
            path_index_count: 0,                   // Set to 0 initially
            pre_path: pre_path,
            post_path: post_path,
            original_path: original_path,
            final_path: Vec::new(),
            do_final_path_subsitution: false, // Set to false initially
            path_lang: item_map
                .get("pathLang")
                .unwrap_or(&Value::String(String::from("")))
                .clone(),
            replacement_path: item_map
                .get("replacementPath")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            method: item_map
                .get("method")
                .unwrap_or(&Value::String(String::from("")))
                .clone(),
            reason: Value::String(String::from("")), // Set to empty string initially
            result_type: Vec::new(), // Set to an empty Vec<Option<ResultType>> initially
            redaction_type: None,    // Set to None initially
        };

        // Check if the "name" field is an object
        if let Some(Value::Object(name_map)) = item_map.get("name") {
            // If the "name" field contains a "description" or "type" field, use it to replace the "name" field in the RedactedObject
            if let Some(name_value) = name_map.get("description").or_else(|| name_map.get("type")) {
                redacted_object.name = name_value.clone();
            }
        }

        // Check if the "reason" field is an object
        if let Some(Value::Object(reason_map)) = item_map.get("reason") {
            // If the "reason" field contains a "description" or "type" field, use it to replace the "reason" field in the RedactedObject
            if let Some(reason_value) = reason_map
                .get("description")
                .or_else(|| reason_map.get("type"))
            {
                redacted_object.reason = reason_value.clone();
            }
        }

        // this has to happen here, before everything else
        redacted_object = set_result_type_from_json_path(v.clone(), &mut redacted_object);

        // set the redaction type
        if let Some(method) = redacted_object.method.as_str() {
            // we don't just assume you are what you say you are...
            match method {
                "emptyValue" => {
                    if !redacted_object.result_type.is_empty() {
                        // I have relaxed the rules around this one, so we can have partialValue as well counts, so if some has inadvertently added a partialValue to an emptyValue, it will still be redacted
                        if redacted_object.result_type.iter().all(|result_type| {
                            matches!(
                                result_type,
                                Some(ResultType::StringNoValue)
                                    | Some(ResultType::EmptyString)
                                    | Some(ResultType::PartialString)
                            )
                        }) {
                            redacted_object.redaction_type = Some(RedactionType::EmptyValue);
                        } else {
                            redacted_object.redaction_type = Some(RedactionType::Unknown);
                        }
                    } else {
                        redacted_object.redaction_type = Some(RedactionType::Unknown);
                    }
                }
                "partialValue" => {
                    if !redacted_object.result_type.is_empty() {
                        if redacted_object.result_type.iter().all(|result_type| {
                            // matches!(result_type, Some(ResultType::PartialString))
                            matches!(
                                result_type,
                                Some(ResultType::StringNoValue)
                                    | Some(ResultType::EmptyString)
                                    | Some(ResultType::PartialString)
                            )
                        }) {
                            redacted_object.redaction_type = Some(RedactionType::PartialValue);
                        } else {
                            redacted_object.redaction_type = Some(RedactionType::Unknown);
                        }
                    } else {
                        redacted_object.redaction_type = Some(RedactionType::Unknown);
                    }
                }
                "replacementValue" => {
                    if !redacted_object.result_type.is_empty() {
                        if redacted_object.result_type.iter().all(|result_type| {
                            matches!(result_type, Some(ResultType::PartialString))
                        }) {
                            // if redacted_object.pre_path.is_some()
                            //     && !redacted_object.pre_path.as_ref().unwrap().is_empty()
                            //     || redacted_object.post_path.is_some()
                            //         && !redacted_object.post_path.as_ref().unwrap().is_empty()
                            if redacted_object.replacement_path.is_some() && !redacted_object.replacement_path.as_ref().unwrap().is_empty()
                                && (redacted_object.pre_path.is_some() && !redacted_object.pre_path.as_ref().unwrap().is_empty()
                                    || redacted_object.post_path.is_some() && !redacted_object.post_path.as_ref().unwrap().is_empty())
                            {
                                redacted_object.redaction_type =
                                    Some(RedactionType::ReplacementValue);
                            } else if redacted_object.replacement_path.is_none() 
                            && (redacted_object.pre_path.is_some() && !redacted_object.pre_path.as_ref().unwrap().is_empty()
                            || redacted_object.post_path.is_some() && !redacted_object.post_path.as_ref().unwrap().is_empty())
                            {
                                redacted_object.redaction_type =
                                    Some(RedactionType::PartialValue); // this logic is really a partial value
                            } else {
                                redacted_object.redaction_type = Some(RedactionType::Unknown);
                            }
                        } else {
                            redacted_object.redaction_type = Some(RedactionType::Unknown);
                        }
                    } else {
                        redacted_object.redaction_type = Some(RedactionType::Unknown);
                    }
                }
                "removal" => {
                    if !redacted_object.result_type.is_empty() {
                        if redacted_object
                            .result_type
                            .iter()
                            .all(|result_type| matches!(result_type, Some(ResultType::Removed)))
                        {
                            redacted_object.redaction_type = Some(RedactionType::Removal);
                        } else {
                            redacted_object.redaction_type = Some(RedactionType::Unknown);
                        }
                    } else {
                        redacted_object.redaction_type = Some(RedactionType::Unknown);
                    }
                }
                _ => {
                    redacted_object.redaction_type = Some(RedactionType::Unknown);
                }
            }
        } else {
            redacted_object.redaction_type = Some(RedactionType::Unknown);
        }

        // now we need to check if we need to do the final path substitution
        match redacted_object.redaction_type {
            // if you are changing what your going to subsitute on, you need to change this.
            Some(RedactionType::EmptyValue)
            | Some(RedactionType::PartialValue)
            | Some(RedactionType::ReplacementValue) => {
                redacted_object.do_final_path_subsitution = true;
            }
            _ => {
                redacted_object.do_final_path_subsitution = false;
            }
        }

        result.push(redacted_object);
    }

    result
}

// this gets us multiple paths, 3 of them!
// $.entities[*].vcardArray[1][?(@[0]=='adr')][3][3]
// this one gets us 3 paths that are arrays
// $.entities[*].vcardArray[1][?(@[0]=='adr')][3]
pub fn set_result_type_from_json_path(u: Value, item: &mut RedactedObject) -> RedactedObject {
    if let Some(path) = item.original_path.as_deref() {
        let path = path.trim_matches('"'); // Remove double quotes
        match JsonPathInst::from_str(path) {
            Ok(json_path) => {
                let finder = JsonPathFinder::new(Box::new(u.clone()), Box::new(json_path));
                let matches = finder.find_as_path();

                if let Value::Array(paths) = matches {
                    if paths.is_empty() {
                        item.result_type.push(Some(ResultType::Removed));
                    } else {
                        // get the length of paths
                        let len = paths.len();
                        println!("PXP Found {} paths", len);
                        // set the path_index_length to the length of the paths
                        item.path_index_count = len as i32;
                        dbg!(&paths);
                        for path_value in paths {
                            if let Value::String(found_path) = path_value {
                                item.final_path.push(Some(found_path.clone())); // Push found_path to final_path on the redacted object
                                let no_value = Value::String("NO_VALUE".to_string());
                                let re = Regex::new(r"\.\[|\]").unwrap();
                                let json_pointer = found_path
                                    .trim_start_matches('$')
                                    .replace('.', "/")
                                    .replace("['", "/")
                                    .replace("']", "")
                                    .replace('[', "/")
                                    .replace(']', "")
                                    .replace("//", "/");
                                let json_pointer = re.replace_all(&json_pointer, "/").to_string();
                                let value_at_path = u.pointer(&json_pointer).unwrap_or(&no_value);
                                if value_at_path.is_string() {
                                    let str_value = value_at_path.as_str().unwrap_or("");
                                    if str_value == "NO_VALUE" {
                                        item.result_type.push(Some(ResultType::StringNoValue));
                                        println!("!! Value at path is NO_VALUE");
                                    } else if str_value.is_empty() {
                                        item.result_type.push(Some(ResultType::EmptyString));
                                        println!("!! Value at path is an empty string");
                                    } else {
                                        item.result_type.push(Some(ResultType::PartialString));
                                        println!("!! Value at path is a string: {}", str_value);
                                    }
                                } else if value_at_path.is_null() {
                                    println!("!! Value at path is null");
                                    item.result_type.push(Some(ResultType::FoundNull));
                                } else if value_at_path.is_array() {
                                    println!("!! Value at path is an array");
                                    item.result_type.push(Some(ResultType::Array));
                                } else if value_at_path.is_object() {
                                    println!("!! Value at path is an object");
                                    item.result_type.push(Some(ResultType::Object));
                                } else {
                                    println!("!! Value at path is not a string - FoundNothing");
                                    item.result_type.push(Some(ResultType::FoundNothing));
                                }
                            } else {
                                println!("!! Value at path is not a string - FoundUnknown");
                                item.result_type.push(Some(ResultType::FoundUnknown));
                            }
                        }
                    }
                } else {
                    println!("!! Finder.find_as_path() returned a bad value");
                    item.result_type
                        .push(Some(ResultType::FoundPathReturnedBadValue));
                }
            }
            Err(e) => {
                println!("Failed to parse JSON path '{}': {}", path, e);
            }
        }
    }
    item.clone()
}

pub fn check_valid_json_path(u: Value, path: &str) -> bool {
    match JsonPathInst::from_str(path) {
        Ok(json_path) => {
            let finder = JsonPathFinder::new(Box::new(u.clone()), Box::new(json_path));
            let matches = finder.find_as_path();

            if let Value::Array(paths) = matches {
                if !paths.is_empty() {
                    for path_value in paths {
                        if let Value::String(found_path) = path_value {
                            let no_value = Value::String("NO_VALUE".to_string());
                            let re = Regex::new(r"\.\[|\]").unwrap();
                            let json_pointer = found_path
                                .trim_start_matches('$')
                                .replace('.', "/")
                                .replace("['", "/")
                                .replace("']", "")
                                .replace('[', "/")
                                .replace(']', "")
                                .replace("//", "/");
                            let json_pointer = re.replace_all(&json_pointer, "/").to_string();
                            let value_at_path = u.pointer(&json_pointer).unwrap_or(&no_value);
                            if value_at_path.is_string() {
                                let str_value = value_at_path.as_str().unwrap_or("");
                                if str_value == "NO_VALUE" {
                                    // This is where Empty1 would be
                                    return true;
                                } else if str_value.is_empty() {
                                    // This is where Empty2 would be
                                    return true;
                                } else {
                                    // This is where Replaced1 would be
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
            false
        }
        Err(_) => false,
    }
}

fn process_redacted_file(file_path: &str) -> Result<(), Box<dyn Error>> {
    #[allow(unused_variables)]
    let mut file = File::open(file_path).map_err(|e| format!("File not found: {}", e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let mut v: Value = serde_json::from_str(&contents).unwrap();

    // if there are any redactions we need to do some modifications
    if let Some(redacted_array) = v["redacted"].as_array() {
        let result = parse_redacted_array(&v, redacted_array);
        // dbg!(&result);
        for redacted_object in result {
            println!("Processing redacted_object...");
            dbg!(&redacted_object);
            if redacted_object.do_final_path_subsitution {
                println!("final_path_exists is true");
                if !redacted_object.final_path.is_empty() {
                    let path_count = redacted_object.path_index_count as usize;
                    for path_index_count in 0..path_count {
                        let final_path_option = &redacted_object.final_path[path_index_count];
                        let result_type = &redacted_object.result_type[path_index_count];
                        println!(
                            "Processing final_path: {:?}, result_type: {:?}",
                            final_path_option, result_type
                        );
                        if let Some(final_path) = final_path_option {
                            println!("Found final_path: {}", final_path);
                            dbg!(final_path);
                            if let Some(redaction_type) = &redacted_object.redaction_type {
                                if *redaction_type == RedactionType::ReplacementValue {
                                    println!("we have a replacement value");
                                    // dbg!(&redacted_object);

                                    let final_path_str = final_path
                                        .trim_start_matches('$')
                                        .replace('.', "/")
                                        .replace("['", "/")
                                        .replace("']", "")
                                        .replace('[', "/")
                                        .replace(']', "")
                                        .replace("//", "/");
                                    dbg!(&final_path_str);

                                    let _final_value = match v.pointer(&final_path_str) {
                                        Some(value) => value.clone(),
                                        None => {
                                            println!("CONTINUE b/c final_path not found");
                                            continue;
                                        }
                                    };

                                    // Unwrap final_path and replacement_path to get a String and then get a reference to the String to get a &str
                                    let final_path = redacted_object
                                        // .final_path
                                        .final_path[path_index_count]
                                        .as_ref()
                                        .expect("final_path is None")
                                        .trim_start_matches('$')
                                        .replace('.', "/")
                                        .replace("['", "/")
                                        .replace("']", "")
                                        .replace('[', "/")
                                        .replace(']', "")
                                        .replace("//", "/");
                                    let replacement_path = redacted_object
                                        .replacement_path
                                        .as_ref()
                                        .expect("replacement_path is None");

                                    dbg!(&final_path);
                                    dbg!(&replacement_path);

                                    // Get the value at final_path
                                    let final_value = v
                                        .pointer(&final_path)
                                        .expect("final_path not found")
                                        .clone();

                                    match replace_with(v.clone(), replacement_path, &mut |_| {
                                        Some(final_value.clone())
                                    }) {
                                        Ok(new_v) => {
                                            v = new_v;
                                            println!("Replaced value at replacement_path");
                                        }
                                        Err(e) => {
                                            println!(
                                                "Failed to replace value at replacement_path: {}",
                                                e
                                            )
                                        }
                                    } // end match replace_with
                                } else if *redaction_type == RedactionType::EmptyValue
                                    || *redaction_type == RedactionType::PartialValue
                                {
                                    println!("we have an empty or partial value");
                                    // dbg!(&redacted_object);

                                    let final_path_str = final_path
                                    .trim_start_matches('$')
                                    .replace('.', "/")
                                    .replace("['", "/")
                                    .replace("']", "")
                                    .replace('[', "/")
                                    .replace(']', "")
                                    .replace("//", "/");
                                    dbg!(&final_path_str);

                                    // You may want to replace with a different value for these types
                                    let final_value = match v.pointer(&final_path_str) {
                                        Some(value) => {
                                            println!("Pointer Found value: {:?}", value);
                                            value.clone()
                                        }
                                        None => {
                                            println!("CONTINUE b/c final_path not found");
                                            continue;
                                        }
                                    };

                                    match replace_with(v.clone(), final_path, &mut |x| {
                                        println!("Replacing value...");
                                        if x.is_string() {
                                            match x.as_str() {
                                                Some("") => {
                                                    println!("Value is an empty string");
                                                    Some(json!("*REDACTED*"))
                                                }
                                                Some(s) => {
                                                    println!("Value is a string: {}", s);
                                                    Some(json!(format!("*{}*", s)))
                                                }
                                                _ => {
                                                    println!("Value is a non-string");
                                                    Some(json!("*REDACTED*"))
                                                }
                                            }
                                        } else if x.is_null() {
                                            println!("Value is null");
                                            Some(final_value.clone())
                                        } else if x.is_boolean() {
                                            println!("Value is a boolean");
                                            Some(final_value.clone())
                                        } else if x.is_number() {
                                            println!("Value is a number");
                                            Some(final_value.clone())
                                        } else if x.is_array() {
                                            println!("Value is an array");
                                            Some(final_value.clone())
                                        } else if x.is_object() {
                                            println!("Value is an object");
                                            Some(final_value.clone())
                                        } else {
                                            println!("Value is not a string");
                                            Some(final_value.clone())
                                        }
                                    }) {
                                        Ok(new_v) => {
                                            v = new_v;
                                            println!("Replaced value at empty or partial path");
                                        }

                                        _ => {
                                            println!(
                                                "Failed to replace value at empty or partial path - WHY?"
                                            );
                                            v = v;
                                        } // Err(e) => {
                                          //     println!("Failed to replace value at empty or partial path: {}", e);
                                          //     // we have to keep going, don't do anything with it
                                          //     v = v;
                                          // }
                                    } // end match replace_with
                                    println!("End match replace with...");
                                } else {
                                    println!("other type of object - we did nothing with it");
                                } // end if replacementValue
                                  // You can now use result_type here
                                println!("Result type: {:?}", result_type);
                            } // end if redaction_type
                        } // end if final_path_option
                    }
                } // end !redacted_object.final_path.is_empty()
            } // end if we are doing final_path_subsitution or not
        } // end for each redacted_object
    } // END if there are redactions

    // convert v back into json
    println!("Converting v back into JSON...");
    let json = serde_json::to_string_pretty(&v).unwrap();
    println!("{}", json);
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::ffi::OsStr;

    #[test]
    fn test_process_directory() {
        // Specify the directory path
        // let dir_path = "path/to/your/directory";
        let dir_path = std::env::var("REDACTED_EXAMPLES").expect("REDACTED_EXAMPLES not set");
        println!("Directory path: {}", dir_path);

        // Read the directory
        if let Ok(entries) = fs::read_dir(dir_path) {
            println!("Reading directory");
            // Iterate over each entry
            for entry in entries {
                if let Ok(entry) = entry {
                    // If the entry is a file
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            // Get the file path
                            let file_path = entry.path();
                            // Check if the file extension is .json
                            if file_path.extension().and_then(OsStr::to_str) == Some("json") {
                                let file_name = file_path.file_name().unwrap().to_str().unwrap();
                                println!("Processing file: {}", file_name);
                                let file_path_str = file_path.to_str().unwrap();

                                // Process the file
                                let result = process_redacted_file(file_path_str);
                                assert!(
                                    result.is_ok(),
                                    "Failed to process file: {}",
                                    file_path_str
                                );
                            }
                        }
                    }
                }
            }
        } else {
            println!("Failed to read directory");
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    use std::fs;
    use std::ffi::OsStr;

    // simple test replacement for the moment
    // test the empty value
    // process_redacted_file("test_files/example-1_empty_value.json")?;
    // test the partial value
    // process_redacted_file("test_files/example-2_partial_value.json")?;
    // test the replacement value
    process_redacted_file("test_files/example-3_replacement_value.json")?;
    // test the removal value


    // let dir_path = std::env::var("REDACTED_EXAMPLES").expect("REDACTED_EXAMPLES not set");
    // println!("Directory path: {}", dir_path);
    
    // // Read the directory
    // if let Ok(entries) = fs::read_dir(dir_path) {
    //     println!("Reading directory");
    //     // Iterate over each entry
    //     for entry in entries {
    //         if let Ok(entry) = entry {
    //             // If the entry is a file
    //             if let Ok(metadata) = entry.metadata() {
    //                 if metadata.is_file() {
    //                     // Get the file path
    //                     let file_path = entry.path();
    //                     // Check if the file extension is .json
    //                     if file_path.extension().and_then(OsStr::to_str) == Some("json") {
    //                         let file_name = file_path.file_name().unwrap().to_str().unwrap();
    //                         println!("Processing file: {}", file_name);
    //                         let file_path_str = file_path.to_str().unwrap();
    
    //                         // Process the file
    //                         let result = process_redacted_file(file_path_str);
    //                         assert!(
    //                             result.is_ok(),
    //                             "Failed to process file: {}",
    //                             file_path_str
    //                         );
    
    //                         // Prompt the user
    //                         println!("Do you want to continue? (yes/no)");
    //                         let mut answer = String::new();
    //                         std::io::stdin().read_line(&mut answer).unwrap();
    //                         if answer.trim() == "no" {
    //                             std::process::exit(0);
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // } else {
    //     println!("Failed to read directory");
    // }
    Ok(())
}
