#![allow(dead_code)]
#![allow(unused_imports)]
extern crate json_value_merge;
// mod shadow;

use json_value_merge::Merge;
use jsonpath_lib::select;
use serde_json::{json, Value, Map};
use std::error::Error;
use std::str::FromStr;
extern crate jsonpath_lib as jsonpath;
use jsonpath::replace_with;
use jsonpath_rust::{JsonPathFinder, JsonPathInst};
use regex::Regex;
use std::collections::HashMap;

use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub struct RedactedObject {
  pub name: Value,
  pub pre_path: Value,
  pub path_lang: Value,
  pub replacement_path: Value,
  pub method: Value,
  pub reason: Value,
  pub result_type: Option<ResultType>,
}

// use crate::shadow::*;

// These are the different types of results that we can get from the JSON path checks
#[derive(Debug, PartialEq)]
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

pub fn check_json_paths2(u: Value, data: Vec<RedactedObject>) -> Vec<RedactedObject> {
  let mut results = Vec::new();

  for mut item in data {
      let path = item.pre_path.as_str().unwrap_or(item.replacement_path.as_str().unwrap()).trim_matches('"'); // Remove double quotes
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

// pub fn check_json_paths2(u: Value, data: Vec<HashMap<String, String>>) -> Vec<HashMap<String, String>> {
//   let mut results = Vec::new();

//   for mut item in data {
//       let path = item.get("prePath").or_else(|| item.get("postPath")).unwrap().trim_matches('"'); // Remove double quotes
//       match JsonPathInst::from_str(path) {
//           Ok(json_path) => {
//               let finder = JsonPathFinder::new(Box::new(u.clone()), Box::new(json_path));
//               let matches = finder.find_as_path();

//               if let Value::Array(paths) = matches {
//                   if paths.is_empty() {
//                       item.insert("result".to_string(), "Removed1".to_string());
//                   } else {
//                       for path_value in paths {
//                           if let Value::String(found_path) = path_value {
//                               let no_value = Value::String("NO_VALUE".to_string());
//                               let re = Regex::new(r"\.\[|\]").unwrap();
//                               let json_pointer = found_path
//                                   .trim_start_matches('$')
//                                   .replace('.', "/")
//                                   .replace("['", "/")
//                                   .replace("']", "")
//                                   .replace('[', "/")
//                                   .replace(']', "")
//                                   .replace("//", "/");
//                               let json_pointer = re.replace_all(&json_pointer, "/").to_string();
//                               let value_at_path = u.pointer(&json_pointer).unwrap_or(&no_value);
//                               if value_at_path.is_string() {
//                                   let str_value = value_at_path.as_str().unwrap_or("");
//                                   if str_value == "NO_VALUE" {
//                                       item.insert("result".to_string(), "Empty1".to_string());
//                                   } else if str_value.is_empty() {
//                                       item.insert("result".to_string(), "Empty2".to_string());
//                                   } else {
//                                       item.insert("result".to_string(), "Replaced1".to_string());
//                                   }
//                               } else if value_at_path.is_null() {
//                                   item.insert("result".to_string(), "Removed2".to_string());
//                               } else if value_at_path.is_array() {
//                                   item.insert("result".to_string(), "Replaced2".to_string());
//                               } else if value_at_path.is_object() {
//                                   item.insert("result".to_string(), "Replaced3".to_string());
//                               } else {
//                                   item.insert("result".to_string(), "Removed3".to_string());
//                               }
//                           } else {
//                               item.insert("result".to_string(), "Removed4".to_string());
//                           }
//                       }
//                   }
//               } else {
//                   item.insert("result".to_string(), "Removed5".to_string());
//               }
//           }
//           Err(e) => {
//               println!("Failed to parse JSON path '{}': {}", path, e);
//           }
//       }
//       results.push(item);
//   }
//   results
// }


// Returns a Vector of tuples with (ResultType, path_it_is_supposed_to_be_at, path_where_it_is_found)
pub fn check_json_paths(u: Value, paths: Vec<String>) -> Vec<(ResultType, String, String)> {
  let mut results = Vec::new();

  for path in paths {
      let path = path.trim_matches('"'); // Remove double quotes
      match JsonPathInst::from_str(path) {
          Ok(json_path) => {
              let finder = JsonPathFinder::new(Box::new(u.clone()), Box::new(json_path));
              let matches = finder.find_as_path();

              if let Value::Array(paths) = matches {
                  if paths.is_empty() {
                      results.push((ResultType::Removed1, path.to_string(), "".to_string()));
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
                                      results.push((
                                          ResultType::Empty1,
                                          path.to_string(),
                                          found_path,
                                      ));
                                  } else if str_value.is_empty() {
                                      results.push((
                                          ResultType::Empty2,
                                          path.to_string(),
                                          found_path,
                                      ));
                                  } else {
                                      results.push((
                                          ResultType::Replaced1,
                                          path.to_string(),
                                          found_path,
                                      ));
                                  }
                              } else if value_at_path.is_null() {
                                  results.push((
                                      ResultType::Removed2,
                                      path.to_string(),
                                      found_path,
                                  ));
                              } else if value_at_path.is_array() {
                                  results.push((
                                      ResultType::Replaced2,
                                      path.to_string(),
                                      found_path,
                                  ));
                              } else if value_at_path.is_object() {
                                  results.push((
                                      ResultType::Replaced3,
                                      path.to_string(),
                                      found_path,
                                  ));
                              } else {
                                  results.push((
                                      ResultType::Removed3,
                                      path.to_string(),
                                      found_path,
                                  ));
                              }
                          } else {
                              results.push((
                                  ResultType::Removed4,
                                  path.to_string(),
                                  "".to_string(),
                              ));
                          }
                      }
                  }
              } else {
                  results.push((ResultType::Removed5, path.to_string(), "".to_string()));
              }
          }
          Err(e) => {
              println!("Failed to parse JSON path '{}': {}", path, e);
          }
      }
  }
  // dbg!(&results);
  results
}


// Returns all the json paths in the object
pub fn get_all_paths_for_object(
  obj: &Value,
  current_path: String,
) -> Vec<(String, Value, String)> {
  match obj {
      Value::Object(map) => {
          let mut paths = vec![];
          for (key, value) in map {
              let new_path = if current_path.is_empty() {
                  format!("$.{}", key)
              } else {
                  format!("{}.{}", current_path, key)
              };
              // dbg!(&key, &value, &new_path);
              paths.push((key.clone(), value.clone(), new_path.clone()));
              paths.extend(get_all_paths_for_object(value, new_path));
          }
          paths
      }
      Value::Array(arr) => arr
          .iter()
          .enumerate()
          .flat_map(|(i, value)| {
              let new_path = format!("{}[{}]", current_path, i);
              get_all_paths_for_object(value, new_path)
          })
          .collect(),
      _ => vec![],
  }
}

// pull the JSON paths from prePath and postPath
pub fn get_pre_and_post_paths(paths: Vec<(String, Value, String)>) -> Vec<String> {
  paths
      .into_iter()
      .filter(|(key, _, _)| key == "prePath" || key == "postPath")
      .filter_map(|(_, value, _)| value.as_str().map(|s| s.to_string()))
      .collect()
}

// fn parse_redacted_array(redacted_array: &Vec<Value>) -> Vec<HashMap<String, String>> {
//   let mut result: Vec<HashMap<String, String>> = Vec::new();

//   for item in redacted_array {
//       let mut map: HashMap<String, String> = HashMap::new();
//       for (key, value) in item.as_object().unwrap() {
//           match value {
//               Value::Object(inner_map) => {
//                   if let Some(inner_value) = inner_map.get("description") {
//                       map.insert(key.clone(), inner_value.as_str().unwrap().to_string());
//                   }
//               },
//               _ => {
//                   map.insert(key.clone(), value.as_str().unwrap().to_string());
//               }
//           }
//       }
//       result.push(map);
//   }

//   result
// }

fn parse_redacted_array(redacted_array: &Vec<Value>) -> Vec<RedactedObject> {
  let mut result: Vec<RedactedObject> = Vec::new();

  for item in redacted_array {
      let item_map = item.as_object().unwrap();
      let mut redacted_object = RedactedObject {
          name: Value::String(String::from("")), // Set to empty string initially
          pre_path: item_map.get("prePath").unwrap_or(&Value::String(String::from(""))).clone(),
          path_lang: item_map.get("pathLang").unwrap_or(&Value::String(String::from(""))).clone(),
          replacement_path: item_map.get("replacementPath").unwrap_or(&Value::String(String::from(""))).clone(),
          method: item_map.get("method").unwrap_or(&Value::String(String::from(""))).clone(),
          reason: item_map.get("reason").unwrap_or(&Value::String(String::from(""))).clone(),
          result_type: None, // Set to None initially
      };

      // Check if the "name" field is an object
      if let Some(Value::Object(name_map)) = item_map.get("name") {
          // If the "name" field contains a "description" or "type" field, use it to replace the "name" field in the RedactedObject
          if let Some(name_value) = name_map.get("description").or_else(|| name_map.get("type")) {
              redacted_object.name = name_value.clone();
          }
      }

      result.push(redacted_object);
  }

  result
}


fn main() -> Result<(), Box<dyn Error>> {
  #[allow(unused_variables)]

  let redacted_file = "/Users/adam/Dev/json_path_match/test_files/wrong.json";
  let mut file = File::open(redacted_file).expect("File not found");
  let mut contents = String::new();
  file.read_to_string(&mut contents).expect("Failed to read file");

  let v: Value = serde_json::from_str(&contents).unwrap();
  let redacted_array = v["redacted"].as_array().unwrap();
  let result = parse_redacted_array(redacted_array);
  dbg!(&result);

 let all_paths: Vec<(String, Value, String)> = get_all_paths_for_object(&v, "".to_string());
//  dbg!(&all_paths);

 let all_redacted_paths: Vec<String> = get_pre_and_post_paths(all_paths);
 /* &all_redacted_paths = [
    "$.lunarNIC_harshMistressNotes",
    "$.entities[?(@.roles[0]=='registrant')].vcardArray[1][?(@[0]=='fn')][3]",
] */
  dbg!(&all_redacted_paths);

  let mut to_change = check_json_paths(v.clone(), all_redacted_paths.into_iter().collect());
  /* &to_change = [
    (
        Replaced2,
        "$.lunarNIC_harshMistressNotes",
        "$.['lunarNIC_harshMistressNotes']",
    ),
    (
        Removed1,
        "$.entities[?(@.roles[0]=='registrant')].vcardArray[1][?(@[0]=='fn')][3]",
        "",
    ),
] */
  dbg!(&to_change);

  let d = check_json_paths2(v.clone(), result);
  dbg!(&d);

  println!("Hello, world!");
  Ok(())
}
