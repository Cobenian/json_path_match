//
// json searializes the object
// the returns a vector with pairs
// for each 'key' in the field
// [ 
//     ('key-name', 'json-path'),
//     ('key-name', 'json-path'),
//     ('key-name', 'json-path'),
//     ('key-name', 'json-path'),
//     ('key-name', 'json-path'),
// ]    
// fun get_kp_json_paths_for_object(obj1) {

// }

// I need a function called get_kp_json_paths_for_object that returns a vector with pairs for each 'key' in the json object and the json path to it, it should look something like this and then I need it pretty-printed in the main function
//
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


// Does the same as above but for the redaction object
// fun get_redacted_kp_json_paths_for_object(obj1) {

// }
// then we have to check if any of the key-names match
// if they do, then have to see if the paths are _equivalent_
// that's the real trick! How do we determine equivalence?
// What if there is more than one kp match

// returns boolean
// fun is_equal_json_path_for_objects(obj1, ob2) {

// }

// does a step through of the whole object
// reporting on filters as we go
// fun debug_step_through_json_paths_for_object() {
// // hopefully we don't need this
// }

use serde_json::{json, Value};
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use jsonpath_rust::{JsonPathFinder, JsonPathQuery, JsonPathInst, JsonPathValue};
use std::str::FromStr;
// use std::collections::HashMap;
use std::collections::HashSet;

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
    let filtered_paths: Vec<_> = paths.iter()
    .filter(|(key, _, _)| key == "prePath" || key == "postPath")
    .collect();

    let mut unique_filtered_paths = Vec::new();
    let mut seen = HashSet::new();
    
    for path in filtered_paths {
        if seen.insert(path.1.as_str().unwrap().to_string()) {
            unique_filtered_paths.push(path.clone());
        }
    }
    
    // Now unique_filtered_paths contains unique paths based on the `value` field.
    let mut idx = 0;
    for (_key, value, _path) in &unique_filtered_paths {
        idx += 1;
        println!("{:02}: -> ('{}')", idx, value); 
    }

    println!("=====================================");

    // this works well
    let mut cidx = 0;
    for (_key, value, _path) in &unique_filtered_paths {
        // println!("checking jsonPath: {}", value);
        let json_path = JsonPathInst::from_str(value.as_str().unwrap()).unwrap();
        let finder = JsonPathFinder::new(Box::new(u.clone()), Box::new(json_path));
        
        // let matches = finder.find_slice();
        // for val in matches {
        //     println!("{:?}", val);
        // }

        // Works great
        // let matches = finder.find_slice();
        // for val in matches {
        //     cidx += 1;
        //     match val {
        //         JsonPathValue::Slice(v, _path) => {
        //             println!("{:02}: {:?} -> {}  ", cidx, v, value);
        //         }
        //         _ => {
        //             println!("{:02}: REM -> {}", cidx, value);
        //         }
        //     }
        // }
        let matches = finder.find_slice();
        for val in matches {
            cidx += 1;
            match val {
                JsonPathValue::Slice(v, _path) => {
                    if v.is_string() && v.as_str().unwrap_or("") == "" {
                        println!("{:02}: EMP -> {}  ", cidx, value);
                    } else {
                        println!("{:02}: {:?} -> {}  ", cidx, v, value);
                    }
                }
                _ => {
                    println!("{:02}: REM -> {}", cidx, value);
                }
            }
        }

    }
    // for (_key, value, _path) in &filtered_paths {
    //     println!("checking jsonPath: {}", value);
    //     let json_path = JsonPathInst::from_str(value.as_str().unwrap()).unwrap();
    //     let finder = JsonPathFinder::new(Box::new(u.clone()), Box::new(json_path));
    
    //     let matches = finder.find_slice();
    //     for val in matches {
    //         match val {
    //             JsonPathValue::Slice(v, path) => {
    //                 let key = path.split('.').last().unwrap_or("");
    //                 println!("Key: {}, Value: {:?}", key, v);
    //             }
    //             _ => {}
    //         }
    //     }
    //}


    Ok(())
}

// So I think this is how we go with this
// We are able to walk a tree of JSON paths
// when we have filter expressions that causes problems.
// We have to chunk each part of the json path that
// has filter expressions, building a non-filtering path
// as we go .. a 'clean-path' obejct. When we hit a filter, we 
// must check to see if that object has any of the 
// 'filters', add a regalar json path equivalent to 
// to the the clean-path-object and keep going down
// Then it would be possible to compare a regular path
// that serde generates and the filter BS.
//
// references: 
//  https://github.com/json-path/JsonPath
//  https://www.baeldung.com/guide-to-jayway-jsonpath
// !!! This one looks like it works !!!
// https://crates.io/crates/jsonpath-rust
//
// This means we wouldn't have to do the above
// and can only worry about the four methods:
//
// 1. Redaction by Removal Method (! this is the issue ! See Below !)
// 2. Redaction by Empty Value Method 
// 3. Redaction by Partial Value Method
// 4. Redaction by Replacement Value Method

// The other problem with the removal of objects is
// that we need to build a tree of the object itself.
// When we walk a tree along with the path, step-by-step
// we mark objects as 'good' for their existence.
// When the redaction path claims there is an object that
// doesn't exist in the tree then we have to mark the node
// at that level, or one level up as dirty.
// Therefore we could mark that node in the display as
// having a redaction in the value of it.
// But we have to able to do the top walking first.
// If we can't get the filter-path equivalent then
// we won't be able to get a clean-path-object-tree
// because we'll never be able to walk the tree itself

// The other idea is that json-filter-path expressions
// aren't allowed at all. You have to explicitly 'mark'(?)
// them out as regular json paths.
