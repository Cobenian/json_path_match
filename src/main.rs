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
use jsonpath::replace_with;
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

fn parse_redacted_array(redacted_array: &Vec<Value>) -> Vec<RedactedObject> {
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
        let mut redacted_object = RedactedObject {
            name: Value::String(String::from("")), // Set to empty string initially
            pre_path: pre_path.clone(),
            path_lang: item_map
                .get("pathLang")
                .unwrap_or(&Value::String(String::from("")))
                .clone(),
            replacement_path: item_map
                .get("replacementPath")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            post_path: post_path.clone(),
            method: item_map
                .get("method")
                .unwrap_or(&Value::String(String::from("")))
                .clone(),
            reason: Value::String(String::from("")), // Set to empty string initially
            result_type: None,                       // Set to None initially
            final_path: pre_path.or(post_path),       // Set to pre_path if it's set, otherwise set to post_path
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

        result.push(redacted_object);
    }

    result
}

pub fn check_json_paths(u: Value, data: Vec<RedactedObject>) -> Vec<RedactedObject> {
    let mut results = Vec::new();

    for mut item in data {
        let path = item
            .pre_path
            .as_deref()
            .unwrap_or(item.replacement_path.as_deref().unwrap())
            .trim_matches('"'); // Remove double quotes
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
        results.push(item);
    }
    results
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
    let redacted_file = "/Users/adam/Dev/json_path_match/test_files/wrong.json";
    let mut file = File::open(redacted_file).expect("File not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read file");

    let mut v: Value = serde_json::from_str(&contents).unwrap();
    let redacted_array = v["redacted"].as_array().unwrap();

    // if there are any redactions we need to do some modifications
    if let Some(redacted_array) = v["redacted"].as_array() {
        let result = parse_redacted_array(redacted_array);
        dbg!(&result);

        // Check the JSON paths
        // let validated_results = check_json_paths(v.clone(), result.clone());
        // dbg!(&validated_results);

        // Get the paths that we need to redact, the pre and post paths but not the replacementValue ones
        let _pre_and_post_paths: Vec<&str> = result
            .iter()
            .filter(|item| item.method != Value::String("replacementValue".to_string()))
            .filter_map(|item| {
                item.pre_path
                    .as_deref()
                    .or_else(|| item.post_path.as_deref())
            })
            .collect();

        // foreach of those, replace them with the *REDACTED* value
        // for path in pre_and_post_paths {
        //     let json_path = path;
        //     match replace_with(v.clone(), json_path, &mut |v| match v.as_str() {
        //         Some("") => Some(json!("*REDACTED*")),
        //         Some(s) => Some(json!(format!("*{}*", s))),
        //         _ => Some(json!("*REDACTED*")),
        //     }) {
        //         Ok(val) => {
        //             v = val; // No need to declare `v` as mutable again
        //         },
        //         Err(e) => {
        //             eprintln!("Error replacing value: {}", e);
        //         }
        //     }
        // }

        // find all the replacementValues and replace them with the value in the replacementPath
        let _replacement_value_paths: Vec<&str> = result
            .iter()
            .filter(|item| item.method == Value::String("replacementValue".to_string()))
            .filter_map(|item| {
                item.pre_path
                    .as_deref()
                    .or_else(|| item.post_path.as_deref())
            })
            .collect();
    }

    println!("Hello, world!");
    Ok(())
}
