use jsonpath_rust::{JsonPathFinder, JsonPathInst, JsonPathValue};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;

// fn check_json_paths(u: Value, paths: Vec<String>) -> Vec<(&'static str, String, String)> {
//   let mut results = Vec::new();

//   for path in paths {
//       let finder = JsonPathFinder::new(Box::new(u.clone()), Box::new(JsonPathInst::from_str(&path).unwrap()));
//       let matches = finder.find_as_path();

//       if let Value::Array(paths) = matches {
//           for path_value in paths {
//               if let Value::String(found_path) = path_value {
//                   let value_at_path = u.pointer(&found_path.replace("$", "")).unwrap_or(&Value::Null);
//                   if value_at_path.is_string() && value_at_path.as_str().unwrap_or("") == "" {
//                       results.push(("empty", path.clone(), found_path));
//                   } else {
//                       results.push(("replaced", path.clone(), found_path));
//                   }
//               } else {
//                   results.push(("removed", path.clone(), "".to_string()));
//               }
//           }
//       }
//   }

//   results
// }


// fn check_json_paths(u: Value, paths: Vec<String>) -> Vec<(&'static str, String, String)> {
//   let mut results = Vec::new();

//   for path in paths {
//       match JsonPathInst::from_str(&path) {
//           Ok(json_path) => {
//               let finder = JsonPathFinder::new(Box::new(u.clone()), Box::new(json_path));
//               let matches = finder.find_as_path();

//               if let Value::Array(paths) = matches {
//                   for path_value in paths {
//                       if let Value::String(found_path) = path_value {
//                           let value_at_path = u.pointer(&found_path.replace("$", "")).unwrap_or(&Value::Null);
//                           if value_at_path.is_string() && value_at_path.as_str().unwrap_or("") == "" {
//                               results.push(("empty", path.clone(), found_path));
//                           } else {
//                               results.push(("replaced", path.clone(), found_path));
//                           }
//                       } else {
//                           results.push(("removed", path.clone(), "".to_string()));
//                       }
//                   }
//               }
//           }
//           Err(e) => {
//               println!("Failed to parse JSON path '{}': {}", path, e);
//           }
//       }
//   }

//   results
// }
fn check_json_paths(u: Value, paths: Vec<String>) -> Vec<(&'static str, String, String)> {
  let mut results = Vec::new();

  for path in paths {
      let path = path.trim_matches('"'); // Remove double quotes
      match JsonPathInst::from_str(path) {
          Ok(json_path) => {
            println!("json_path: {:?}", path);
              let finder = JsonPathFinder::new(Box::new(u.clone()), Box::new(json_path));
              let matches = finder.find_as_path();

              if let Value::Array(paths) = matches {
                // print the length of matches
                println!("\t\tmatches: {:?}", paths.len());
                  if paths.is_empty() {
                      results.push(("removed", path.to_string(), "".to_string()));
                  } else {
                      for path_value in paths {
                          if let Value::String(found_path) = path_value {
                              let no_value = Value::String("NO_VALUE".to_string());
                              let value_at_path = u.pointer(&found_path.replace("$", "")).unwrap_or(&no_value);
                              if value_at_path.is_string() {
                                  let str_value = value_at_path.as_str().unwrap_or("");
                                  if str_value == "NO_VALUE" {
                                      results.push(("empty", path.to_string(), found_path));
                                  } else if str_value.is_empty() {
                                      results.push(("empty", path.to_string(), found_path));
                                  } else {
                                      results.push(("replaced", path.to_string(), found_path));
                                  }
                              } else {
                                  results.push(("removed2", path.to_string(), found_path));
                              }
                          } else {
                              results.push(("removed3", path.to_string(), "".to_string()));
                          }
                      }
                  }
              } else {
                  results.push(("removed4", path.to_string(), "".to_string()));
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
      Value::Array(arr) => {
          arr.iter().enumerate().flat_map(|(i, value)| {
              let new_path = format!("{}[{}]", current_path, i);
              get_kp_json_paths_for_object(value, new_path)
          }).collect()
      }
      _ => vec![],
  }
}


fn main() -> Result<(), Box<dyn Error>> {
  // Open the file in read-only mode with buffer.
  let file = File::open("/Users/adam/Dev/json_path_match/test_files/lookup_with_redaction.json")?;
  let reader = BufReader::new(file);
  // Read the JSON contents of the file as an instance of `serde_json::Value`.
  let u: Value = serde_json::from_reader(reader)?;

  let paths = get_kp_json_paths_for_object(&u, "".to_string());
  for (key, value, path) in &paths {
      println!("('{}', '{}', '{}')", key, value, path);
  }
  // Do something with `u` here...
  let filtered_paths: Vec<_> = paths.iter()
  .filter(|(key, _, _)| key == "prePath" || key == "postPath")
  .collect();

  let mut json_paths_to_redacted_objects = Vec::new();
  let mut seen = HashSet::new();
  
  for path in filtered_paths {
      if seen.insert(path.1.as_str().unwrap().to_string()) {
          json_paths_to_redacted_objects.push(path.clone());
      }
  }
  
  // These are the filter paths taken out from the redacted object
  // Now unique_filtered_paths contains unique paths based on the `value` field.
    let mut idx = 0;
    for (key, value, path) in &json_paths_to_redacted_objects {
      idx += 1;
      println!("{:02}: -> value: ('{}') -> key: {} -> path: {}", idx, value, key, path);
      // println!("{:02}: -> ('{}')", idx, value); 
    }

    println!("=====================================");

    // now get JUST the paths that redaction talks about
    let filtered_paths: Vec<String> = json_paths_to_redacted_objects.iter()
    .map(|(_value, key, _path)| key.to_string())
    .collect();

    
    for filtered_path in &filtered_paths {
      println!("filtered_path: {}", filtered_path);
    } 

    println!("=====================================");
    
    // Now we need to check those paths against the JSON object
    let checks = check_json_paths(u, filtered_paths.into_iter().map(|s| s.into()).collect());
    for (status, path, found_path) in checks {
        println!("{}: {} -> {}", status, path, found_path);
    } 
  Ok(())
}