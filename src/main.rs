#![allow(dead_code)]
#![allow(unused_imports)]
extern crate json_value_merge;
// mod shadow;

use json_value_merge::Merge;
use jsonpath_lib::select;
use serde_json::{json, Value};
use std::error::Error;
use std::str::FromStr;
extern crate jsonpath_lib as jsonpath;
use jsonpath::replace_with;
use jsonpath_rust::{JsonPathFinder, JsonPathInst};
use regex::Regex;

use std::fs::File;
use std::io::Read;

// use crate::shadow::*;

#[derive(Debug, PartialEq)]
pub enum ResultType {
    Removed1,
    Empty1,
    Empty2,
    Replaced1,
    Removed2,
    Replaced2,
    Replaced3,
    Removed3,
    Removed4,
    Removed5,
}


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


// Checks the redaction in the object and returns the json paths that we need
pub fn get_redacted_paths_for_object(
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
              paths.extend(get_redacted_paths_for_object(value, new_path));
          }
          paths
      }
      Value::Array(arr) => arr
          .iter()
          .enumerate()
          .flat_map(|(i, value)| {
              let new_path = format!("{}[{}]", current_path, i);
              get_redacted_paths_for_object(value, new_path)
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


fn main() -> Result<(), Box<dyn Error>> {
  #[allow(unused_variables)]

  let redacted_file = "/Users/adam/Dev/json_path_match/test_files/wrong.json";
  let mut file = File::open(redacted_file).expect("File not found");
  let mut contents = String::new();
  file.read_to_string(&mut contents).expect("Failed to read file");

  let v: Value = serde_json::from_str(&contents).unwrap();

 let all_paths: Vec<(String, Value, String)> = get_redacted_paths_for_object(&v, "".to_string());
 let all_redacted_paths: Vec<String> = get_pre_and_post_paths(all_paths);

  dbg!(&all_redacted_paths);

  let mut to_change = check_json_paths(v.clone(), all_redacted_paths.into_iter().collect());
  dbg!(&to_change);
  
  println!("Hello, world!");
  Ok(())
}
