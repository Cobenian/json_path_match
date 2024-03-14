use jsonpath_rust::{JsonPathFinder, JsonPathInst, JsonPathValue};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;

// VERSION ONE - NO KEY NAME
fn check_json_paths(u: Value, paths: Vec<String>) -> Vec<(&'static str, String)> {
    let mut results = Vec::new();

    for path in paths {
        let json_path = JsonPathInst::from_str(&path).unwrap();
        let finder = JsonPathFinder::new(Box::new(u.clone()), Box::new(json_path));
        let matches = finder.find_slice();

        for val in matches {
            match val {
                JsonPathValue::Slice(v, _path) => {
                    if v.is_string() && v.as_str().unwrap_or("") == "" {
                        results.push(("empty", path.clone()));
                    } else {
                        results.push(("replaced", path.clone()));
                    }
                }
                _ => {
                    results.push(("removed", path.clone()));
                }
            }
        }
    }

    results
}

// fn pretty_print_and_count(results: Vec<(&'static str, String)>) -> HashMap<&'static str, usize> {
//     let mut counts = HashMap::new();

//     for (status, path) in results {
//         println!("{}: {}", status, path);
//         *counts.entry(status).or_insert(0) += 1;
//     }

//     counts
// }

// VERSION TWO - WITH KEY NAME
fn check_json_paths(
    u: Value,
    keys_and_paths: Vec<(String, String)>,
) -> Vec<(&'static str, String, String)> {
    let mut results = Vec::new();

    for (key, path) in keys_and_paths {
        let json_path = JsonPathInst::from_str(&path).unwrap();
        let finder = JsonPathFinder::new(Box::new(u.clone()), Box::new(json_path));
        let matches = finder.find_slice();

        for val in matches {
            match val {
                JsonPathValue::Slice(v, _path) => {
                    if v.is_string() && v.as_str().unwrap_or("") == "" {
                        results.push(("empty", key.clone(), path.clone()));
                    } else {
                        results.push(("replaced", key.clone(), path.clone()));
                    }
                }
                _ => {
                    results.push(("removed", key.clone(), path.clone()));
                }
            }
        }
    }

    results
}

fn pretty_print_and_count(
    results: Vec<(&'static str, String, String)>,
) -> HashMap<&'static str, usize> {
    let mut counts = HashMap::new();

    for (status, key_name, path) in results {
        println!("{}: {} -> {}", status, key_name, path);
        *counts.entry(status).or_insert(0) += 1;
    }

    counts
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

fn main() -> Result<(), Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open("/Users/adam/Dev/json_path_match/test_files/lookup_with_redaction.json")?;
    let reader = BufReader::new(file);
    // Read the JSON contents of the file as an instance of `serde_json::Value`.
    let u: Value = serde_json::from_reader(reader)?;

    let paths = get_kp_json_paths_for_object(&u, "".to_string());
    for (key, value, path) in &paths {
        // println!("('{}', '{}', '{}')", key, value, path);
    }
    // Do something with `u` here...
    let filtered_paths: Vec<_> = paths
        .iter()
        .filter(|(key, _, _)| key == "prePath" || key == "postPath")
        .collect();

    let mut unique_filtered_paths = Vec::new();
    let mut seen = HashSet::new();

    for path in filtered_paths {
        if seen.insert(path.1.as_str().unwrap().to_string()) {
            unique_filtered_paths.push(path.clone());
        }
    }

    // Extract the key names and the paths from each tuple in unique_filtered_paths
    let extracted_keys_and_paths: Vec<(String, String)> = unique_filtered_paths
        .iter()
        .map(|(key, value, _)| (key.clone(), value.clone().as_str().unwrap().to_string()))
        .collect();

    
    // now do something with it
    let checks = check_json_paths(u, extracted_keys_and_paths);
    pretty_print_and_count(checks);

    Ok(())
}
