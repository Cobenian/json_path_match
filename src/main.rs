#![allow(dead_code)]
#![allow(unused_imports)]
extern crate json_value_merge;
// mod shadow;

use json_value_merge::Merge;
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
    pub pre_path: Option<String>,
    pub post_path: Option<String>,
    pub final_path: Option<String>,
    pub final_path_exists: bool,
    pub path_lang: Value,
    pub replacement_path: Option<String>,
    pub method: Value,
    pub reason: Value,
    pub result_type: Option<ResultType>,
}
// use crate::shadow::*;

// These are the different types of results that we can get from the JSON path checks
#[derive(Debug, PartialEq, Clone)]
pub enum ResultType {
    Empty1, // (*) what we found in the value paths array was a string but has no value (yes, this is a little weird, but does exist) `Redaction by Empty Value`
    Empty2, // (*) what we found in the value paths array was a string but it is an empty string `Redaction by Empty Value`
    Replaced1, // (*) what we found in the value paths array was a string and it does have a value `Redaction by Partial Value` and/or `Redaction by Replacement Value`
    Replaced2, // what we found in the value paths array was _another_ array (have never found this)
    Replaced3, // what we found in the value paths array was an object (have never found this)
    Removed1, // (*) paths array is empty, finder.find_as_path() found nothing `Redaction by Removal`
    Removed2, // value in paths array is null (have never found this)
    Removed3, // fall through, value in paths array is not anything else (have never found this)
    Removed4, // what we found was not a JSON::Value::string (have never found this)
    Removed5, // what finder.find_as_path() returned was not a Value::Array (have never found this, could possibly be an error)
}

// fn replace_with_ng(json: &mut Value, path: &str, new_value: &str) -> Result<(), Box<dyn std::error::Error>> {
//     let mut selector = Selector::new();
//     selector.str_path(path)?;
//     for value in selector.select(json)? {
//         *value = Value::String(new_value.to_string());
//     }
//     Ok(())
// }

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
        let final_path = pre_path.clone().or(post_path.clone());
        let mut redacted_object = RedactedObject {
            name: Value::String(String::from("")), // Set to empty string initially
            pre_path: pre_path,
            post_path: post_path,
            final_path: final_path.clone(),
            final_path_exists: false, // Set to false initially
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
            result_type: None,                       // Set to None initially
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

        // here is our sanity checking
        if let Some(_final_path_str) = final_path {
            redacted_object = set_result_type_from_json_path(v.clone(), redacted_object);
            match redacted_object.result_type {
                // if you are changing what is considered a "valid" path, you need to change this
                Some(ResultType::Empty1) | Some(ResultType::Empty2) | Some(ResultType::Replaced1) => {
                    redacted_object.final_path_exists = true;
                }
                _ => {
                    redacted_object.final_path_exists = false;
                }
            }
        }

        result.push(redacted_object);
    }

    result
}

pub fn set_result_type_from_json_path(u: Value, mut item: RedactedObject) -> RedactedObject {
    if let Some(path) = item.final_path.as_deref() {
        let path = path.trim_matches('"'); // Remove double quotes
        match JsonPathInst::from_str(path) {
            Ok(json_path) => {
                let finder = JsonPathFinder::new(Box::new(u.clone()), Box::new(json_path));
                let matches = finder.find_as_path();

                if let Value::Array(paths) = matches {
                    if paths.is_empty() {
                        item.result_type = Some(ResultType::Removed1);
                    } else {
                        for path_value in paths {
                            if let Value::String(found_path) = path_value {
                                item.final_path = Some(found_path.clone()); // Assign found_path to final_path
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
                                        item.result_type = Some(ResultType::Empty1);
                                    } else if str_value.is_empty() {
                                        item.result_type = Some(ResultType::Empty2);
                                    } else {
                                        item.result_type = Some(ResultType::Replaced1);
                                    }
                                } else if value_at_path.is_null() {
                                    item.result_type = Some(ResultType::Removed2);
                                } else if value_at_path.is_array() {
                                    item.result_type = Some(ResultType::Replaced2);
                                } else if value_at_path.is_object() {
                                    item.result_type = Some(ResultType::Replaced3);
                                } else {
                                    item.result_type = Some(ResultType::Removed3);
                                }
                            } else {
                                item.result_type = Some(ResultType::Removed4);
                            }
                        }
                    }
                } else {
                    item.result_type = Some(ResultType::Removed5);
                }
            }
            Err(e) => {
                println!("Failed to parse JSON path '{}': {}", path, e);
            }
        }
    }
    item
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
        },
        Err(_) => false,
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    #[allow(unused_variables)]
    // let redacted_file = "/Users/adam/Dev/json_path_match/test_files/wrong.json";
    let redacted_file = "/Users/adam/Dev/json_path_match/test_files/with_all_fields_lookup_redaction.json";
    // let redacted_file = "/Users/adam/Dev/json_path_match/test_files/simple_example_domain_w_redaction.json";
    let mut file = File::open(redacted_file).expect("File not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read file");

    let mut v: Value = serde_json::from_str(&contents).unwrap();
    let redacted_array = v["redacted"].as_array().unwrap();

    // if there are any redactions we need to do some modifications
    if let Some(redacted_array) = v["redacted"].as_array() {
        let result = parse_redacted_array(&v, redacted_array);
        dbg!(&result);


        for redacted_object in result {
            println!("Processing redacted_object...");
            if redacted_object.final_path_exists {
                println!("final_path_exists is true");
                if let Some(final_path) = redacted_object.final_path {
                    println!("Found final_path: {}", final_path);
                    dbg!(&final_path);
                    // remove the $ from the final_path
                    // let final_path = final_path.trim_matches('$');
                    // let json_path = path;
                    // dbg!(&json_path);
                    // match replace_with(v.clone(), json_path, &mut |v| match v.as_str() {
                    //     Some("") => Some(json!("*REDACTED*")),
                    //     Some(s) => Some(json!(format!("*{}*", s))),
                    //     _ => Some(json!("*REDACTED*")),
                    // }) {
                    //     Ok(val) => v = val,
                    //     Err(e) => {
                    //         eprintln!("Error replacing value: {}", e);
                    //     }
                    // }
                    match replace_with(v.clone(), &final_path, &mut |v| {
                        println!("Replacing value...");
                        if v.is_string() {
                            match v.as_str() {
                                Some("") => {
                                    println!("Value is an empty string");
                                    Some(json!("*REDACTED*"))
                                },
                                Some(s) => {
                                    println!("Value is a string: {}", s);
                                    Some(json!(format!("*{}*", s)))
                                },
                                _ => {
                                    println!("Value is a non-string");
                                    Some(json!("*REDACTED*"))
                                },
                            }
                        } else {
                            // Handle non-string values here
                            println!("Value is not a string");
                            Some(json!("*NON-STRING VALUE*"))
                        }
                    }) {
                        Ok(val) => {
                            println!("Successfully replaced value");
                            v = val; // No need to declare `v` as mutable again
                        },
                        Err(e) => {
                            println!("Error replacing value: {}", e);
                        }
                    }
                }
            } else {
                println!("final_path_exists is false");
            }
        }

        // find all the replacementValues and replace them with the value in the replacementPath
        // let _replacement_value_paths: Vec<&str> = result
        //     .iter()
        //     .filter(|item| item.method == Value::String("replacementValue".to_string()))
        //     .filter_map(|item| {
        //         item.pre_path
        //             .as_deref()
        //             .or_else(|| item.post_path.as_deref())
        //     })
        //     .collect();
    }

    // convert v back into json
    let json = serde_json::to_string_pretty(&v).unwrap();
    println!("{}", json);
    println!("Hello, world!");
    Ok(())
}
