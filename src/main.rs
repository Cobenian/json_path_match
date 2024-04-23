#![allow(dead_code)]
#![allow(unused_imports)]
extern crate json_value_merge;
mod shadow;
// use crate::shadow::ShadowBuilder;

use json_value_merge::Merge;
use jsonpath_lib::select;
use serde_json::{json, Value};
use std::error::Error;
use std::str::FromStr;

use crate::shadow::{ShadowBuilder, make_shadow_links};
// use serde_json::Value;

enum PathChunk {
    Key(String),
    Index(usize),
}

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

fn chunk_json_path(path: &str) -> Vec<PathChunk> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    let mut chars = path.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '.' => {
                if !current_chunk.is_empty() {
                    chunks.push(PathChunk::Key(current_chunk));
                    current_chunk = String::new();
                }
            }
            '[' => {
                if !current_chunk.is_empty() {
                    chunks.push(PathChunk::Key(current_chunk));
                    current_chunk = String::new();
                }
                while let Some(&c) = chars.peek() {
                    if c == ']' {
                        chars.next();
                        break;
                    } else {
                        current_chunk.push(chars.next().unwrap());
                    }
                }
                if let Ok(index) = current_chunk.parse::<usize>() {
                    chunks.push(PathChunk::Index(index));
                }
                current_chunk = String::new();
            }
            _ => current_chunk.push(c),
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(PathChunk::Key(current_chunk));
    }

    chunks
}

fn fill_arrays(value: &mut Value) {
    match value {
        Value::Array(arr) => {
            if let Some(first) = arr.first() {
                if first.is_object() && arr.len() < 254 {
                    let template = first.clone();
                    while arr.len() < 254 {
                        arr.push(template.clone());
                    }
                } else {
                    for item in arr.iter_mut() {
                        fill_arrays(item);
                    }
                }
            }
        }
        Value::Object(obj) => {
            for (_, v) in obj.iter_mut() {
                fill_arrays(v);
            }
        }
        _ => {}
    }
}

fn create_shadow_object_of_domain() -> Value {
    let mut shadow_data: Value = serde_json::from_str(
        r#"
      {
        "objectClassName" : "domain",
        "handle" : "*REDACTED*",
        "ldhName" : "*REDACTED*",
        "nameservers" :
        [
          {
            "objectClassName" : "*REDACTED*",
            "ldhName" : "*REDACTED*"
          },
          {
            "objectClassName" : "*REDACTED*",
            "ldhName" : "*REDACTED*"
          }
        ],
        "secureDNS":
        {
          "delegationSigned": true,
          "dsData":
          [
            {
              "keyTag": null,
              "algorithm": null,
              "digestType": null,
              "digest": "*REDACTED*"
            }
          ]
        },
        "remarks" :
        [
          {
            "description" :
            [
              "*REDACTED*",
              "*REDACTED*"
            ]
          }
        ],
        "links" :
        [
          {
            "value": "*REDACTED*",
            "rel" : "*REDACTED*",
            "href" : "*REDACTED*",
            "type" : "*REDACTED*"
      
          }
        ],
        "events" :
        [
          {
            "eventAction" : "*REDACTED*",
            "eventDate" : "*REDACTED*"
          },
          {
            "eventAction" : "*REDACTED*",
            "eventDate" : "*REDACTED*",
            "eventActor" : "*REDACTED*"
          }
        ],
        "entities" :
        [
          {
            "objectClassName" : "*REDACTED*",
            "handle" : "*REDACTED*",
            "vcardArray":[
              "vcard",
              [
                ["version", {}, "text", "4.0"],
                ["fn", {}, "text", "*REDACTED*"],
                ["kind", {}, "text", "individual"],
                ["lang", {
                  "pref":"1"
                }, "language-tag", "fr"],
                ["lang", {
                  "pref":"2"
                }, "language-tag", "en"],
                ["org", {
                  "type":"*REDACTED*"
                }, "text", "Example"],
                ["title", {}, "text", "*REDACTED*"],
                ["role", {}, "text", "*REDACTED*"],
                ["adr",
                  { "type":"work" },
                  "text",
                  [
                    "*REDACTED*",
                    "*REDACTED*",
                    "*REDACTED*",
                    "*REDACTED*",
                    "*REDACTED*",
                    "*REDACTED*",
                    "*REDACTED*"
                  ]
      
                ],
                ["tel",
                  { "type":["work", "voice"], "pref":"1" },
                  "uri", "*REDACTED*"
                ],
                ["email",
                  { "type":"work" },
                  "text", "*REDACTED*"
                ]
              ]
            ],
            "roles" : [ "*REDACTED*" ],
            "remarks" :
            [
              {
                "description" :
                [
                  "*REDACTED*",
                  "*REDACTED*"
                ]
              }
            ],
            "links" :
            [
              {
                "value": "*REDACTED*",
                "rel" : "*REDACTED*",
                "href" : "*REDACTED*",
                "type" : "*REDACTED*"
              }
            ],
            "events" :
            [
              {
                "eventAction" : "*REDACTED*",
                "eventDate" : "*REDACTED*"
              },
              {
                "eventAction" : "*REDACTED*",
                "eventDate" : "*REDACTED*",
                "eventActor" : "*REDACTED*"
              }
            ]
          }
        ],
        "network" :
        {
          "objectClassName" : "*REDACTED*",
          "handle" : "*REDACTED*",
          "startAddress" : "*REDACTED*",
          "endAddress" : "*REDACTED*",
          "ipVersion" : "*REDACTED*",
          "name": "*REDACTED*",
          "type" : "*REDACTED*",
          "country" : "*REDACTED*",
          "parentHandle" : "*REDACTED*",
          "status" : [ "*REDACTED*" ]
        }
      }
"#,
    )
    .unwrap();

    fill_arrays(&mut shadow_data);

    shadow_data
}

fn merge_path(
    redacted_data: &mut Value,
    shadow_data: &Value,
    path: &str,
) -> Result<(), Box<dyn Error>> {
    if path == "$" {
        *redacted_data = shadow_data.clone();
        return Ok(());
    }

    let merge_object = match select(shadow_data, path)?.pop() {
        Some(value) => value,
        None => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "No value found at the given JSON path",
            )))
        }
    };

    let path_chunks = chunk_json_path(path);

    let mut current_value = redacted_data;
    for chunk in &path_chunks {
        match chunk {
            PathChunk::Key(key) => {
                if current_value.is_object() {
                    let current_value_clone = current_value.clone();
                    current_value = match current_value.get_mut(key) {
                        Some(value) => value,
                        None => {
                            eprintln!(
                                "Error: Key '{}' not found in JSON object: {:?}",
                                key, current_value_clone
                            );
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "Key not found in JSON object",
                            )));
                        }
                    };
                } else {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Invalid JSON path",
                    )));
                }
            }
            PathChunk::Index(index) => {
                if current_value.is_array() {
                    current_value = &mut current_value[*index];
                } else {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Invalid JSON path",
                    )));
                }
            }
        }
    }

    *current_value = merge_object.clone();

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    #[allow(unused_variables)]
    // let redacted_file =
    //     "/Users/adam/Dev/json_path_match/test_files/example_domain_obejct_with_redaction.json";
    // let shadow_file =
    //     "/Users/adam/Dev/json_path_match/test_files/shadow_example_domain_object.json";

    // let redacted_data = std::fs::read_to_string(redacted_file)?;
    // let mut redacted_data: Value = serde_json::from_str(&redacted_data)?;

    // let shadow_data = std::fs::read_to_string(shadow_file)?;
    // let shadow_data: Value = serde_json::from_str(&shadow_data)?;
    // // let merge_object = select(&shadow_data, "$.secureDNS.dsData[0].keyTag")?.pop().unwrap();
    // let paths = ["$.secureDNS.dsData[0].keyTag", "$.network.handle"];

    // for path in &paths {
    //     merge_path(&mut redacted_data, &shadow_data, path)?;
    // }

    // println!("{}", serde_json::to_string_pretty(&redacted_data)?);

    // dbg!(&redacted_data);

    // let fake_shadow_obj = create_shadow_object_of_domain();
    // dbg!(&fake_shadow_obj);

    // XXX testing out the shadow object
    // let pretty_json = serde_json::to_string_pretty(&fake_shadow_obj).unwrap();
    // println!("{}", pretty_json);



    // let shadow_domain = ShadowDomainBuilder::new()
    // .domain("some domain".to_string())
    // .build();

    // let shadow_domain_json = serde_json::to_string_pretty(&shadow_domain)?;
    // println!("{}", shadow_domain_json);

    // // let handle = make_shadow_handle();  
    // println!("handle: {}", handle);
    // let handle: serde_json::Value = serde_json::from_str(&handle).unwrap();
    let builder = ShadowBuilder {
      links: make_shadow_links(),
      conformances: Value::Null,
      notices_and_remarks: Value::Null,
      lang: Value::Null,
      events: Value::Null,
      status: Value::Null,
      port43: Value::Null,
      public_ids: Value::Null,
      object_class_name: Value::Null,
    };

    let pretty_builder = serde_json::to_string_pretty(&builder).unwrap();
    println!("{}", pretty_builder);

    Ok(())
}
