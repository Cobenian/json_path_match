#![allow(unused_imports)]
// extern crate jsonpath_lib as jsonpath;
extern crate json_value_merge;
// use jsonpath::replace_with;
// // use jsonpath_rust::{JsonPathFinder, JsonPathInst, JsonPathValue};
// use jsonpath_rust::{JsonPathFinder, JsonPathInst};
// use regex::Regex;
use serde_json::{json, Value};

use json_value_merge::Merge;
use jsonpath_lib::select;
// use json_value_merge::merge;
// use std::{error::Error, str::FromStr};

// use serde::{Deserialize, Serialize};

// use serde_json_diff;
// use serde_json::{json, Value};
// use std::fs::File;
// use std::io::BufReader;
// use jsonpath_rust::{JsonPathFinder, JsonPathQuery, JsonPathInst, JsonPathValue};
use std::str::FromStr;
// // use std::collections::HashMap;
// use std::collections::HashSet;

use std::error::Error;

// fn add_field(json: &mut Value, path: &str, new_value: Value) {
//   let path = path.trim_start_matches("$."); // strip the $. prefix
//   let parts: Vec<&str> = path.split('.').collect();
//   let last = parts.last().unwrap();

//   let mut current = json;

//   for part in &parts[0..parts.len() - 1] {
//       let array_parts: Vec<&str> = part.split('[').collect();
//       if array_parts.len() > 1 {
//           let index = usize::from_str(array_parts[1].trim_end_matches(']')).unwrap();
//           current = &mut current[array_parts[0]][index];
//       } else {
//           current = &mut current[*part];
//       }
//   }

//   if last.contains('[') {
//       let array_parts: Vec<&str> = last.split('[').collect();
//       let index = usize::from_str(array_parts[1].trim_end_matches(']')).unwrap();
//       current[array_parts[0]][index] = new_value;
//   } else {
//       // current[last] = new_value;
//       current[*last] = new_value;
//   }
// }

fn add_field(
    json: &mut Value,
    path: &str,
    new_value: Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = path.trim_start_matches("$."); // strip the $. prefix
    let parts: Vec<&str> = path.split('.').collect();
    let last = parts.last().unwrap();

    let mut current = json;

    for part in &parts[0..parts.len() - 1] {
        let array_parts: Vec<&str> = part.split('[').collect();
        if array_parts.len() > 1 {
            let index = usize::from_str(array_parts[1].trim_end_matches(']'))?;
            current = &mut current[array_parts[0]][index];
        } else {
            current = &mut current[*part];
        }
    }

    if last.contains('[') {
        let array_parts: Vec<&str> = last.split('[').collect();
        let index = usize::from_str(array_parts[1].trim_end_matches(']'))?;
        current[array_parts[0]][index] = new_value;
    } else {
        current[*last] = new_value;
    }

    Ok(())
}
fn chunk_json_path(path: &str) -> Vec<String> {
    let path = path.trim_start_matches('$');
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

fn main() -> Result<(), Box<dyn Error>> {
    #[allow(unused_variables)]
    let redacted_file =
        "/Users/adam/Dev/json_path_match/test_files/example_domain_obejct_with_redaction.json";
    let shadow_file =
        "/Users/adam/Dev/json_path_match/test_files/shadow_example_domain_object.json";

    let redacted_data = std::fs::read_to_string(redacted_file)?;
    let mut redacted_data: Value = serde_json::from_str(&redacted_data)?;

    let shadow_data = std::fs::read_to_string(shadow_file)?;
    let shadow_data: Value = serde_json::from_str(&shadow_data)?;

    // dbg!(&redacted_data);
    // dbg!(&shadow_data);
    // Select the object to merge from the shadow_data
    let merge_object = select(&shadow_data, "$.network.handle")?.pop().unwrap();

    // Get the path chunks
    let path = "$.network.handle";
    let path_chunks = chunk_json_path(path);

    // Manually traverse the redacted_data object and merge the merge_object at the correct location
    let mut current_value = &mut redacted_data;
    for chunk in path_chunks {
      let chunk_clone = chunk.clone(); // clone the chunk value
      dbg!(&chunk); // print the current chunk
      if !current_value[&chunk_clone].is_object() {
          // If the chunk doesn't exist or isn't an object, create a new object at this chunk
          current_value[&chunk_clone] = json!({});
      }
      // Move to the next chunk
      current_value = current_value.get_mut(&chunk_clone).unwrap();
  }

    // Merge the merge_object into the final chunk
    *current_value = merge_object.clone();

    
    dbg!(&redacted_data);

    Ok(())
}
