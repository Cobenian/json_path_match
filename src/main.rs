#[allow(unused_imports)]
extern crate jsonpath_lib as jsonpath;
use jsonpath::replace_with;
// use jsonpath_rust::{JsonPathFinder, JsonPathInst, JsonPathValue};
use jsonpath_rust::{JsonPathFinder, JsonPathInst};
use regex::Regex;
use serde_json::{json, Value};
use std::{error::Error, str::FromStr};

use serde::{Deserialize, Serialize};

use serde_json_diff;
// use serde_json::Value;

// use std::collections::HashMap;

// how do we NOT use this???
// use json::JsonValue;

#[derive(Serialize, Deserialize, Debug)]
struct Store {
    book: Vec<Book>,
    bicycle: Bicycle,
    employees: Vec<Employee>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Book {
    category: String,
    author: String,
    title: String,
    price: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Bicycle {
    color: String,
    price: f64,
    gears: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Employee {
    employee_id: u32,
    name: String,
    department: String,
    roles: Vec<String>,
    projects: Vec<Project>,
    skills: Vec<Skill>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Project {
    project_id: u32,
    name: String,
    start_date: String,
    end_date: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Skill {
    name: String,
    level: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Root {
    store: Store,
}

fn check_json_paths(u: Value, paths: Vec<String>) -> Vec<(&'static str, String, String)> {
    let mut results = Vec::new();

    for path in paths {
        let path = path.trim_matches('"'); // Remove double quotes
        match JsonPathInst::from_str(path) {
            Ok(json_path) => {
                // println!("json_path: {:?}", path);
                let finder = JsonPathFinder::new(Box::new(u.clone()), Box::new(json_path));
                let matches = finder.find_as_path();

                if let Value::Array(paths) = matches {
                    // print the length of matches
                    // println!("\t\tmatches: {:?}", paths.len());
                    if paths.is_empty() {
                        results.push(("REMOVED1", path.to_string(), "".to_string()));
                    } else {
                        for path_value in paths {
                            if let Value::String(found_path) = path_value {
                                let no_value = Value::String("NO_VALUE".to_string());
                                // Convert the JSONPath expression, example: $.['store'].['bicycle'].['color'] to the JSON Pointer /store/bicycle/color and retrieves the value <whatever> at that path in the JSON document.
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
                                        results.push(("EMPTY1", path.to_string(), found_path));
                                    } else if str_value.is_empty() {
                                        results.push(("EMPTY2", path.to_string(), found_path));
                                    // } else if str_value == "" {
                                    //     results.push(("EMPTY3", path.to_string(), found_path));
                                    } else {
                                        results.push(("REPLACED1", path.to_string(), found_path));
                                    }
                                } else if value_at_path.is_null() {
                                    results.push(("REMOVED2", path.to_string(), found_path));
                                } else if value_at_path.is_array() {
                                    results.push(("REPLACED2", path.to_string(), found_path));
                                } else if value_at_path.is_object() {
                                    results.push(("REPLACED3", path.to_string(), found_path));
                                } else {
                                    results.push(("REMOVED3", path.to_string(), found_path));
                                }
                            } else {
                                results.push(("REMOVED4", path.to_string(), "".to_string()));
                            }
                        }
                    }
                } else {
                    results.push(("REMOVED5", path.to_string(), "".to_string()));
                }
            }
            Err(e) => {
                println!("Failed to parse JSON path '{}': {}", path, e);
            }
        }
    }
    results
}

fn get_kp_json_paths_for_object(obj: &Value, current_path: String) -> Vec<(String, Value, String)> {
    match obj {
        Value::Object(map) => {
            let mut paths = vec![];
            for (key, value) in map {
                let new_path = if current_path.is_empty() {
                    format!("$.{}", key)
                } else {
                    format!("{}.{}", current_path, key)
                };
                paths.push((key.clone(), value.clone(), new_path.clone()));
                paths.extend(get_kp_json_paths_for_object(value, new_path));
            }
            paths
        }
        Value::Array(arr) => arr
            .iter()
            .enumerate()
            .flat_map(|(i, value)| {
                let new_path = format!("{}[{}]", current_path, i);
                get_kp_json_paths_for_object(value, new_path)
            })
            .collect(),
        _ => vec![],
    }
}

fn find_paths_to_redact(checks: &[(&str, String, String)]) -> Vec<String> {
    checks
        .iter()
        .filter(|(status, _, _)| matches!(*status, "EMPTY1" | "EMPTY2" | "EMPTY3" | "REPLACED1"))
        .map(|(_, _, found_path)| found_path.clone())
        .collect()
}

// fn add_field(json: &mut JsonValue, path: &str, new_value: JsonValue) {
fn add_field(json: &mut Value, path: &str, new_value: Value) {
    let path = path.trim_start_matches("$."); // strip the $. prefix
    let parts: Vec<&str> = path.split('.').collect();
    let last = parts.last().unwrap();

    let mut current = json;

    for part in &parts[0..parts.len() - 1] {
        let array_parts: Vec<&str> = part.split('[').collect();
        if array_parts.len() > 1 {
            let index = usize::from_str(array_parts[1].trim_end_matches(']')).unwrap();
            current = &mut current[array_parts[0]][index];
        } else {
            current = &mut current[*part];
        }
    }

    if last.contains('[') {
        let array_parts: Vec<&str> = last.split('[').collect();
        let index = usize::from_str(array_parts[1].trim_end_matches(']')).unwrap();
        current[array_parts[0]][index] = new_value;
    } else {
        // current[last] = new_value;
        current[*last] = new_value;
    }
}

fn chunk_json_path(path: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_brackets = false;
    let mut in_parentheses = false;

    for c in path.chars() {
        match c {
            '.' if !in_brackets && !in_parentheses => {
                if !current.is_empty() {
                    parts.push(current.clone());
                }
                current = String::new();
                current.push(c);
            }
            '[' if !in_parentheses => {
                if !current.is_empty() {
                    parts.push(current.clone());
                }
                current = String::new();
                current.push(c);
                in_brackets = true;
            }
            '(' if in_brackets => {
                current.push(c);
                in_parentheses = true;
            }
            ')' if in_parentheses => {
                current.push(c);
                in_parentheses = false;
            }
            ']' if in_brackets && !in_parentheses => {
                current.push(c);
                parts.push(current.clone());
                current = String::new();
                in_brackets = false;
            }
            _ => current.push(c),
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

fn check_json_path_validity(u: Value, path: String) -> bool {
    let path = path.trim_matches('"'); // Remove double quotes
    match JsonPathInst::from_str(&path) {
        Ok(json_path) => {
            let finder = JsonPathFinder::new(Box::new(u.clone()), Box::new(json_path));
            let matches = finder.find_as_path();

            if let Value::Array(paths) = matches {
                if paths.is_empty() {
                    false
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
                                if str_value == "NO_VALUE" || str_value.is_empty() {
                                    return false;
                                } else {
                                    return true;
                                }
                            } else if value_at_path.is_null() {
                                return false;
                            } else {
                                return true;
                            }
                        }
                    }
                    false
                }
            } else {
                false
            }
        }
        Err(e) => {
            println!("Failed to parse JSON path '{}': {}", path, e);
            false
        }
    }
}

// fn apply_diff(mut redacted_data: Value, difference: &Value) -> Value {
//     if let Value::Object(difference_object) = difference {
//         for (key, value) in difference_object {
//             if let Value::Object(value_object) = value {
//                 if let Some(entry_difference) = value_object.get("entry_difference") {
//                     if entry_difference == "extra" || entry_difference == "value" {
//                         if let Value::Object(redacted_object) = &mut redacted_data {
//                             if let Some(value_diff) = value_object.get("value_diff") {
//                                 if let Some(target_value) = value_diff.get("target_value") {
//                                     redacted_object.insert(key.clone(), target_value.clone());
//                                     println!("----------------->>> Inserted: {}", key);
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
//     redacted_data
// }

fn apply_diff(redacted_dat: &mut Value, difference: &Value) {
    if let Value::Object(difference_object) = difference {
        if let Some(different_entries) = difference_object.get("different_entries") {
            if let Value::Object(entries_object) = different_entries {
                for (key, value) in entries_object {
                    if let Value::Object(value_object) = value {
                        if let Some(entry_difference) = value_object.get("entry_difference") {
                            if entry_difference == "extra" || entry_difference == "value" {
                                if let Some(value_diff) = value_object.get("value_diff") {
                                    if let Value::Object(value_diff_object) = value_diff {
                                        if let Some(target_value) =
                                            value_diff_object.get("target_value")
                                        {
                                            // Insert the target_value at the correct nested level in redacted_dat
                                            if let Some(redacted_dat_object) =
                                                redacted_dat.as_object_mut()
                                            {
                                                redacted_dat_object
                                                    .insert(key.to_string(), target_value.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // Ensure key exists in redacted_dat before trying to access it
                        if let Some(redacted_dat_object) = redacted_dat.as_object_mut() {
                            let entry = redacted_dat_object
                                .entry(key.to_string())
                                .or_insert(Value::Null);
                            // Recursively apply the difference to the nested entries
                            apply_diff(entry, &value_object["different_entries"]);
                        }
                    }
                }
            }
        }
        if let Some(different_pairs) = difference_object.get("different_pairs") {
            if let Value::Object(pairs_object) = different_pairs {
                for (key, value) in pairs_object {
                    // Ensure key exists in redacted_dat before trying to access it
                    if let Some(redacted_dat_object) = redacted_dat.as_object_mut() {
                        let entry = redacted_dat_object
                            .entry(key.to_string())
                            .or_insert(Value::Null);
                        // Recursively apply the difference to the nested pairs
                        apply_diff(entry, value);
                    }
                }
            }
        }
    }
}

// this one works but gets ALL they keys of both objects :(
// fn get_keys_to_create(difference: &Value) -> Vec<String> {
//     let mut keys_to_create = Vec::new();

//     if let Value::Object(difference_object) = difference {
//         for (key, value) in difference_object {
//             keys_to_create.push(key.to_string());

//             // If the value is an object, recursively get its keys
//             if let Value::Object(_) = value {
//                 keys_to_create.extend(get_keys_to_create(value));
//             }
//         }
//     }

//     keys_to_create
// }

// another way of getting stuff that sort of works
// fn get_keys_to_create(difference: &Value, path: String) -> Vec<String> {
//     let mut keys_to_create = Vec::new();

//     if let Value::Object(difference_object) = difference {
//         for (key, value) in difference_object {
//             let new_path = if path.is_empty() {
//                 key.to_string()
//             } else {
//                 format!("{}.{}", path, key)
//             };

//             // If the value is an object, recursively get its keys
//             if let Value::Object(value_object) = value {
//                 if let Some(entry_difference) = value_object.get("entry_difference") {
//                     if entry_difference == "missing" || entry_difference == "extra" {
//                         keys_to_create.push(new_path.clone());
//                     }
//                 }
//                 keys_to_create.extend(get_keys_to_create(value, new_path));
//             } else if let Some(entry_difference) = difference_object.get("entry_difference") {
//                 if entry_difference == "missing" || entry_difference == "extra" {
//                     keys_to_create.push(new_path.clone());
//                 }
//             }
//         }
//     }

//     keys_to_create
// }

// more sort of works
// fn get_keys_to_create(difference: &Value, path: Vec<String>) -> Vec<String> {
//     let mut keys_to_create = Vec::new();

//     if let Value::Object(difference_object) = difference {
//         for (key, value) in difference_object {
//             let mut new_path = path.clone();
//             new_path.push(key.to_string());

//             // If the value is an object, recursively get its keys
//             if let Value::Object(value_object) = value {
//                 if let Some(entry_difference) = value_object.get("entry_difference") {
//                     if entry_difference == "missing" || entry_difference == "extra" {
//                         keys_to_create.push(new_path.join("."));
//                     }
//                 }
//                 keys_to_create.extend(get_keys_to_create(value, new_path));
//             }
//         }
//     }

//     keys_to_create
// }

// this is the best one so far
// fn get_keys_to_create(difference: &Value, path: Vec<String>) -> Vec<String> {
//     let mut keys_to_create = Vec::new();

//     match difference {
//         Value::Object(difference_object) => {
//             for (key, value) in difference_object {
//                 let mut new_path = path.clone();
//                 new_path.push(key.to_string());

//                 if let Some(entry_difference) = value.get("entry_difference") {
//                     if entry_difference == "missing" || entry_difference == "extra" {
//                         keys_to_create.push(new_path.join("."));
//                     }
//                 }

//                 if key == "missing_elements" {
//                     keys_to_create.push(new_path.join("."));
//                 }

//                 keys_to_create.extend(get_keys_to_create(value, new_path));
//             }
//         }
//         Value::Array(values) => {
//             for (index, value) in values.iter().enumerate() {
//                 let mut new_path = path.clone();
//                 new_path.push(index.to_string());

//                 keys_to_create.extend(get_keys_to_create(value, new_path));
//             }
//         }
//         _ => {}
//     }

//     keys_to_create
// }


// This one is AWESOME:
// fn get_keys_to_create(difference: &Value, path: Vec<String>) -> Vec<(String, Option<String>)> {
//     let mut keys_to_create = Vec::new();

//     match difference {
//         Value::Object(difference_object) => {
//             for (key, value) in difference_object {
//                 let mut new_path = path.clone();
//                 new_path.push(key.to_string());

//                 if let Some(entry_difference) = value.get("entry_difference") {
//                     if entry_difference == "missing" || entry_difference == "extra" {
//                         keys_to_create.push((new_path.join("."), None));
//                     }
//                 }

//                 if key == "missing_elements" {
//                     keys_to_create.push((new_path.join("."), None));
//                 }

//                 if let Some(difference_of) = value.get("difference_of") {
//                     if difference_of == "scalar" {
//                         if let Some(target) = value.get("target") {
//                             keys_to_create.push((new_path.join("."), Some(target.to_string())));
//                         }
//                     }
//                 }

//                 keys_to_create.extend(get_keys_to_create(value, new_path));
//             }
//         }
//         Value::Array(values) => {
//             for (index, value) in values.iter().enumerate() {
//                 let mut new_path = path.clone();
//                 new_path.push(index.to_string());

//                 keys_to_create.extend(get_keys_to_create(value, new_path));
//             }
//         }
//         _ => {}
//     }

//     keys_to_create
// }

// double-awesome
// fn get_keys_to_create(difference: &Value, path: Vec<String>) -> Vec<(String, Option<String>)> {
//     let mut keys_to_create = Vec::new();

//     match difference {
//         Value::Object(difference_object) => {
//             for (key, value) in difference_object {
//                 let mut new_path = path.clone();
//                 new_path.push(key.to_string());

//                 if let Some(entry_difference) = value.get("entry_difference") {
//                     if entry_difference == "missing" {
//                         let value_to_insert = value.get("value").map(|v| v.to_string());
//                         keys_to_create.push((new_path.join("."), value_to_insert));
//                     }
//                 }

//                 if key == "missing_elements" {
//                     keys_to_create.push((new_path.join("."), None));
//                 }

//                 if let Some(difference_of) = value.get("difference_of") {
//                     if difference_of == "scalar" {
//                         if let Some(target) = value.get("target") {
//                             keys_to_create.push((new_path.join("."), Some(target.to_string())));
//                         }
//                     }
//                 }

//                 keys_to_create.extend(get_keys_to_create(value, new_path));
//             }
//         }
//         Value::Array(values) => {
//             for (index, value) in values.iter().enumerate() {
//                 let mut new_path = path.clone();
//                 new_path.push(index.to_string());

//                 keys_to_create.extend(get_keys_to_create(value, new_path));
//             }
//         }
//         _ => {}
//     }

//     keys_to_create
// }

// triple-awesome
fn get_keys_to_create(difference: &Value, path: Vec<String>) -> Vec<(String, Option<String>)> {
    let mut keys_to_create = Vec::new();

    match difference {
        Value::Object(difference_object) => {
            for (key, value) in difference_object {
                let mut new_path = path.clone();
                new_path.push(key.to_string());

                if let Some(entry_difference) = value.get("entry_difference") {
                    match entry_difference.as_str() {
                        Some("missing") => {
                            let value_to_insert = value.get("value").map(|v| v.to_string());
                            keys_to_create.push((new_path.join("."), value_to_insert));
                        },
                        Some("extra") => {
                            keys_to_create.push((new_path.join("."), Some(format!("\"{}\"", key))));
                        },
                        _ => {}
                    }
                }

                if key == "missing_elements" {
                    keys_to_create.push((new_path.join("."), None));
                }

                if let Some(difference_of) = value.get("difference_of") {
                    if difference_of == "scalar" {
                        if let Some(target) = value.get("target") {
                            keys_to_create.push((new_path.join("."), Some(target.to_string())));
                        }
                    }
                }

                keys_to_create.extend(get_keys_to_create(value, new_path));
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                let mut new_path = path.clone();
                new_path.push(index.to_string());

                keys_to_create.extend(get_keys_to_create(value, new_path));
            }
        }
        _ => {}
    }

    keys_to_create
}

fn main() -> Result<(), Box<dyn Error>> {
    #[allow(unused_variables)]
    //
    let data = r#"
    {
      "store": {
        "book": [
          {
            "category": "",
            "author": "Nigel Rees",
            "title": "Sayings of the Century",
            "price": 8.95
          },
          {
            "category": "fiction",
            "author": "Evelyn Waugh",
            "title": "Sword of Honour",
            "price": 12.99
          }
        ],
        "bicycle": {
          "color": "red",
          "price": 19.95
        },
        "employees": [
          {
            "employee_id": 12345,
            "name": "John Doe",
            "department": "Engineering",
            "roles": ["Software Engineer", "Team Lead"],
            "projects": [
              {
                "project_id": 1,
                "name": "Project A",
                "start_date": "2023-01-01",
                "end_date": "2023-06-30"
              },
              {
                "project_id": 2,
                "name": "Project B",
                "start_date": "2023-07-01",
                "end_date": "2023-12-31"
              }
            ],
            "skills": [
              {
                "name": "Python",
                "level": "Intermediate"
              },
              {
                "name": "JavaScript",
                "level": "Advanced"
              }
            ]
          },
          {
            "employee_id": 12346,
            "name": "Jane Doe",
            "department": "Human Resources",
            "roles": ["HR Manager"],
            "projects": [
              {
                "project_id": 3,
                "name": "Project C",
                "start_date": "2023-01-01",
                "end_date": "2023-12-31"
              }
            ],
            "skills": [
              {
                "name": "Recruiting",
                "level": "Advanced"
              }
            ]
          }
        ]
      }
    }
    "#;

    // let mut v: Value = serde_json::from_str(data)?;
    let v: Value = serde_json::from_str(data)?;

    // Use a JSONPath expression to find the color of the bicycle
    let json_path = "$.store.bicycle.color";
    let mut ret = replace_with(v.clone(), json_path, &mut |_v| Some(json!("blue")))?;

    // println!("{}", serde_json::to_string_pretty(&ret)?);

    let _paths = get_kp_json_paths_for_object(&ret, "".to_string());
    // for (key, value, path) in &paths {
    //     println!("('{}', '{}', '{}')", key, value, path);
    // }
    // println!("========GEARS=============");
    // Use a JSONPath expression to find the "gears" field of the bicycle
    // let json_path = "$.store.bicycle.gears";
    // let ret = replace_with(v.clone(), json_path, &mut |v| {
    //     // If the "gears" field exists, keep its current value
    //     println!("Gears already exits!");
    //     Some(v.clone())
    // });

    // match ret {
    //     Ok(ret) => {
    //         println!("Somehow I made it into here?");
    //         println!("{}", serde_json::to_string_pretty(&ret)?);
    //     }
    //     Err(_) => {
    //         // If the "gears" field does not exist, add it
    //         println!("Gears does not exist!");
    //         if let Some(store) = v.get_mut("store") {
    //             if let Some(bicycle) = store.get_mut("bicycle") {
    //                 if let Some(bicycle_obj) = bicycle.as_object_mut() {
    //                     println!("ADDING GEARS!");
    //                     bicycle_obj.insert("gears".to_string(), json!(5));
    //                 }
    //             }
    //         }
    //         println!("{}", serde_json::to_string_pretty(&v)?);
    //     }
    // }

    // Create a vector of json path strings
    // println!("Checking ....");
    // let json_paths: Vec<String> = ["$.store.bicycle.color", "$.store.bicycle.gears']"];
    let json_paths: Vec<String> = [
        "$.store.bicycle.color",
        "$.store.bicycle.gears",
        "$.store.book[0].category",
        "$.store.book[1].title",
        "$.store",
        "$.store.book[1].price[0].amount[2].nothereatall",
        "$.store.employees[1].skills[0].level",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let checks = check_json_paths(ret.clone(), json_paths.into_iter().collect());
    // let checks = check_json_paths(ret, json_paths.into_iter().map(|s| s.into()).collect());
    // for (status, path, found_path) in &checks {
    // println!(
    //     "STATUS\n {}\nOrigPath\n {}\nFoundPath\n {}\n",
    //     status, path, found_path
    // );
    // }

    // Find the paths to redact
    let redact_paths = find_paths_to_redact(&checks);
    // println!("RedactPaths: {:?}", redact_paths);
    for path in redact_paths {
        let json_path = &path;
        ret = replace_with(ret, json_path, &mut |_v| Some(json!("REDACTED")))?;
    }

    let json_paths: Vec<String> = [
        "$.store.bicycle.color",
        "$.store.bicycle.gears",
        "$.store.book[0].category",
        "$.store.book[1].title",
        "$.store",
        "$.store.book[1].price[0].amount[2].nothereatall",
        "$.store.employees[1].skills[0].level",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let checks = check_json_paths(ret.clone(), json_paths.into_iter().collect());
    // let checks = check_json_paths(ret, json_paths.into_iter().map(|s| s.into()).collect());
    // for (status, path, found_path) in &checks {
    // println!(
    //     "STATUS\n {}\nOrigPath\n {}\nFoundPath\n {}\n",
    //     status, path, found_path
    // );
    // }

    // Find the paths to redact
    let redact_paths = find_paths_to_redact(&checks);
    // println!("RedactPaths: {:?}", redact_paths);
    for path in redact_paths {
        let json_path = &path;
        ret = replace_with(ret, json_path, &mut |_v| Some(json!("REDACTED")))?;
    }

    add_field(
        &mut ret,
        "$.store.bicycle.gears",
        serde_json::Value::Number(serde_json::Number::from(10)),
    );
    // println!("PP:\n{}", serde_json::to_string_pretty(&ret)?);

    // Suck it all back into the structures
    let ret_string = serde_json::to_string(&ret).unwrap();
    // println!("RetString:\n{}", ret_string);
    let data: Root = serde_json::from_str(&ret_string).unwrap();
    // println!("Data is ready... printit");
    // println!("DATA:\n{:#?}", data);

    // Print the color of the bicycle
    // println!("Bicycle color: {}", data.store.bicycle.color);

    // Print the name of the first employee
    if let Some(_first_employee) = data.store.employees.first() {
        // println!("First employee name: {}", first_employee.name);
    }

    // Print the first skill of the last employee
    if let Some(last_employee) = data.store.employees.last() {
        if let Some(_first_skill) = last_employee.skills.first() {
            // println!("First skill of last employee: {}", first_skill.name);
        }
    }

    // Print the category of the first book
    if let Some(_first_book) = data.store.book.first() {
        // println!("Category of the first book: {}", first_book.category);
    }

    // Print the title of the second book
    if let Some(_second_book) = data.store.book.get(1) {
        // println!("Title of the second book: {}", second_book.title);
    }

    let removal_strings = vec![
        "$.entities[?(@.roles[0]=='registrant')].vcardArray[1][?(@[0]=='org')]",
        "$.entities[?(@.roles[0]=='registrant')].vcardArray[1][?(@[0]=='email')]",
        "$.entities[?(@.roles[0]=='registrant')].vcardArray[1][?(@[1].type=='voice')]",
        "$.entities[?(@.roles[0]=='technical')].vcardArray[1][?(@[0]=='email')]",
        "$.entities[?(@.roles[0]=='technical')].vcardArray[1][?(@[1].type=='voice')]",
        "$.entities[?(@.roles[0]=='technical')].vcardArray[1][?(@[1].type=='fax')]",
        "$.entities[?(@.roles[0]=='administrative')]",
        "$.entities[?(@.roles[0]=='billing')]",
    ];

    // suck in the json file: /Users/adam/Dev/json_path_match/test_files/lookup_with_redaction.json
    let redacted_file = "/Users/adam/Dev/json_path_match/test_files/lookup_with_redaction.json";
    let un_redacted_file = "/Users/adam/Dev/json_path_match/test_files/unredacted.json";

    let redacted_data = std::fs::read_to_string(redacted_file)?;
    let mut redacted_data: Value = serde_json::from_str(&redacted_data)?;

    let un_redacted_data = std::fs::read_to_string(un_redacted_file)?;
    let un_redacted_data: Value = serde_json::from_str(&un_redacted_data)?;

    let mut results = Vec::new();

    for rs in removal_strings {
        let chunks = chunk_json_path(&rs);
        let mut current_path = String::new();
        let mut last_good = None;
        let mut first_bad = None;

        for chunk in chunks {
            current_path.push_str(&chunk);

            // Check if the current path is valid
            let is_valid = check_json_path_validity(redacted_data.clone(), current_path.clone());

            if is_valid {
                last_good = Some(current_path.clone());
            } else if first_bad.is_none() {
                first_bad = Some(current_path.clone());
            }
        }

        // Save the last good and the first bad path for this removal string
        results.push((last_good, first_bad));
    }

    // dbg!(results);

    // the diff between the two files
    // println!("DIFF:\n{:#?}", diff(&redacted_data, &un_redacted_data));
    // let difference = diff(&un_redacted_data, &redacted_data);
    // let pretty_difference = serde_json::to_string_pretty(&difference)?;

    // println!("Pretty difference: {}", pretty_difference);

    // let difference = serde_json_diff::values(redacted_data, un_redacted_data);
    // // let difference = serde_json_diff::diff(&redacted_data, &un_redacted_data);
    // match difference {
    //     Some(diff) => {
    //         let pretty_difference = serde_json::to_string_pretty(&diff)?;
    //         println!("Pretty difference: {}", pretty_difference);
    //     }
    //     None => println!("No difference"),
    // }

    // save a copy of the un_redacted_data
    let un_redacted_data_copy = un_redacted_data.clone();

    // let difference = serde_json_diff::values(redacted_data.clone(), un_redacted_data.clone());
    // if let Some(difference) = difference {
    //     let difference_value = serde_json::to_value(&difference).unwrap();

    //     let pretty_difference = serde_json::to_string_pretty(&difference)?;
    //     println!("Pretty difference: {}", pretty_difference);

    // apply_diff(&mut redacted_data, &difference_value);
    // let another_difference = serde_json_diff::values(un_redacted_data.clone(), redacted_data.clone());
    // // let pretty = serde_json::to_string_pretty(&re_redacted_data)?;
    // // println!("Re-Redacted : {}", pretty);
    // // now we have to check if un_redacted_data is the same as re_redacted_data
    // // let another_difference = serde_json_diff::values(un_redacted_data.clone(), re_redacted_data.clone());
    // // if there is a diff just println! DIFFERENT
    // if let Some(another_difference) = another_difference {
    //     let another_difference_value = serde_json::to_value(&another_difference).unwrap();
    //     let _pretty = serde_json::to_string_pretty(&another_difference_value)?;
    //     // println!("DIFFERENT : {}", pretty);
    //     println!("DIFFERENT");
    //     // if its diffferent then is there a difference between the unredacted and the original unredacted
    //     let un_difference = serde_json_diff::values(un_redacted_data_copy.clone(), un_redacted_data.clone());
    //     if let Some(_un_difference) = un_difference {
    //         println!("UNREDACTS ARE FRICKING DIFFERENT");
    //     } else {
    //         println!("UNREDACTS ARE SAME");
    //     }
    // } else {
    //     println!("SAME");
    // }

    // }
    let difference = serde_json_diff::values(redacted_data.clone(), un_redacted_data.clone());
    let difference_value = match difference {
        Some(diff) => {
            println!("There is a difference.");
            serde_json::to_value(diff).unwrap()
        }
        None => serde_json::Value::Null,
    };

    // println!("Difference value: {}", difference_value);
    // let keys_to_create = get_keys_to_create(&difference_value);
    // println!("Number of keys: {}", keys_to_create.len());
    // for key in keys_to_create {
    //     println!("{}", key);
    // }

    // println!("Difference value: {}", difference_value);
    // let keys_to_create = get_keys_to_create(&difference_value, "".to_string());
    // println!("Number of keys: {}", keys_to_create.len());
    // for key in keys_to_create {
    //     println!("{}", key);
    // }

    println!("Difference value: {}", difference_value);
    let keys_to_create = get_keys_to_create(&difference_value, vec![]);
    println!("Number of keys: {}", keys_to_create.len());
    for key in keys_to_create {
        // println!("{}", key);
        println!("{:?}", key);
    }

    Ok(())
}
