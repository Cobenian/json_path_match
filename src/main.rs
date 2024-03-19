extern crate jsonpath_lib as jsonpath;
use jsonpath::replace_with;
// use jsonpath_rust::{JsonPathFinder, JsonPathInst, JsonPathValue};
use jsonpath_rust::{JsonPathFinder, JsonPathInst};
use regex::Regex;
use serde_json::{json, Value};
use std::{error::Error, str::FromStr};

use serde::{Deserialize, Serialize};
// use std::collections::HashMap;

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
                println!("json_path: {:?}", path);
                let finder = JsonPathFinder::new(Box::new(u.clone()), Box::new(json_path));
                let matches = finder.find_as_path();

                if let Value::Array(paths) = matches {
                    // print the length of matches
                    println!("\t\tmatches: {:?}", paths.len());
                    if paths.is_empty() {
                        results.push(("removed1", path.to_string(), "".to_string()));
                    } else {
                        for path_value in paths {
                            if let Value::String(found_path) = path_value {
                                let no_value = Value::String("NO_VALUE".to_string());
                                // Convert the JSONPath expression, example: $.['store'].['bicycle'].['color'] to the JSON Pointer /store/bicycle/color and retrieves the value <whatever> at that path in the JSON document.
                                let re = Regex::new(r"\.\[|\]").unwrap();
                                let json_pointer = dbg!(found_path
                                    .trim_start_matches('$')
                                    .replace('.', "/")
                                    .replace("['", "/")
                                    .replace("']", "")
                                    .replace('[', "/")
                                    .replace(']', "")
                                    .replace("//", "/"));
                                let json_pointer = re.replace_all(&json_pointer, "/").to_string();
                                let value_at_path =
                                    dbg!(u.pointer(&json_pointer).unwrap_or(&no_value));
                                if value_at_path.is_string() {
                                    let str_value = value_at_path.as_str().unwrap_or("");
                                    if str_value == "NO_VALUE" {
                                        results.push(("empty1", path.to_string(), found_path));
                                    } else if str_value.is_empty() {
                                        results.push(("empty2", path.to_string(), found_path));
                                    } else {
                                        results.push(("replaced", path.to_string(), found_path));
                                    }
                                } else if value_at_path.is_null() {
                                    results.push(("removed2 null", path.to_string(), found_path));
                                } else if value_at_path.is_array() {
                                    results.push(("array replaced", path.to_string(), found_path));
                                } else if value_at_path.is_object() {
                                    results.push(("object replaced", path.to_string(), found_path));
                                } else {
                                    results.push(("wtf removed3", path.to_string(), found_path));
                                }
                            } else {
                                results.push(("plain removed3", path.to_string(), "".to_string()));
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
    //
    let data = r#"
    {
      "store": {
        "book": [
          {
            "category": "reference",
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
    let ret = replace_with(v.clone(), json_path, &mut |_v| Some(json!("blue")))?;

    println!("{}", serde_json::to_string_pretty(&ret)?);

    let paths = get_kp_json_paths_for_object(&ret, "".to_string());
    for (key, value, path) in &paths {
        println!("('{}', '{}', '{}')", key, value, path);
    }
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
    println!("Checking ....");
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
    for (status, path, found_path) in checks {
        println!("=> {}: {} -> {}", status, path, found_path);
    }

    // Suck it all back into the structures
    let ret_string = serde_json::to_string(&ret).unwrap();
    let data: Root = serde_json::from_str(&ret_string).unwrap();

    // Print the color of the bicycle
    println!("Bicycle color: {}", data.store.bicycle.color);

    // Print the name of the first employee
    if let Some(first_employee) = data.store.employees.first() {
        println!("First employee name: {}", first_employee.name);
    }

    // Print the first skill of the last employee
    if let Some(last_employee) = data.store.employees.last() {
        if let Some(first_skill) = last_employee.skills.first() {
            println!("First skill of last employee: {}", first_skill.name);
        }
    }

    Ok(())
}
