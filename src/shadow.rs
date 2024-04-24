// RFC9083 Object Classes
// build a ShadownBuilder that can build a datastructure:
// 1. RDAP Conformance
// 2. Links
// 3. Notices and Remarks
// 4. Language Identifier
// 5. Events
// 6. Status
// 7. Port 43 WHOIS SERVER
// 8. Public IDs
// 9. Object Class Name
// First we need the empty JSON VALUE object
// 1. Add the class name, by passing a string to it, output it
// 2. Add the conformance, output it
// 3. Add the links, output it
// 4. At this point we have a redaction
//    we add a link that was removed by trying json_value_merge
//

#![allow(dead_code)]
#![allow(unused_imports)]
extern crate json_value_merge;

use json_value_merge::Merge;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Serialize, Deserialize)]
pub struct ShadowBuilder {
    pub conformances: Value,
    pub links: Value,
    pub notices_and_remarks: Value,
    pub lang: Value,
    pub events: Value,
    pub status: Value,
    pub port43: Value,
    pub public_ids: Value,
    pub object_class_name: Value,
}

fn make_shadow_nameserver() {}

fn make_shadow_entity() {}

fn make_shadow_ip_network() {}

fn make_shadow_asn() {}


pub fn make_shadow_redacted_string() -> Value {
    Value::String("*REDACTED*".to_string())
}

pub fn make_shadow_redacted_array() -> Value {
    Value::Array(vec![Value::String("*REDACTED*".to_string())])
}
pub fn make_shadow_links() -> Value {
    // Change the return type here
    let json_str = r#"
  [
    {
      "value" : "*REDACTED*",
      "rel" : "*REDACTED*",
      "href" : "*REDACTED*",
      "hreflang" : [ "*REDACTED*"],
      "title" : "*REDACTED*",
      "media" : "*REDACTED*",
      "type" : "*REDACTED*"
    }
  ]
  "#;
    serde_json::from_str(json_str).unwrap() // Parse the JSON string into a Value
}

pub fn make_shadow_notices() -> Value {
    let json_str = r#"
[
  {
    "title" : "*REDACTED*",
    "description" :
    [
      "*REDACTED*"
    ],
    "links" :
    [
      {
        "value" : "*REDACTED*",
        "rel" : "*REDACTED*",
        "type" : "*REDACTED*",
        "href" : "*REDACTED*"
      }
    ]
  }
]
"#;
    serde_json::from_str(json_str).unwrap()
}

pub fn make_shadow_lang() -> Value {
    let json_str = r#""*REDACTED*""#; // double up the quotes to get it to be a JSON Value
    serde_json::from_str(json_str).unwrap()
}


pub fn make_shadow_events() -> Value {
    serde_json::from_str(r#"{ "events" : [ { "eventAction" : "*REDACTED*", "eventActor" : "*REDACTED*", "eventDate" : "*REDACTED*" } ] }"#).unwrap()
}

pub fn make_shadow_publicIds() -> Value {
    serde_json::from_str(r#"{ "publicIds": [ { "type":"*REDACTED*", "identifier":"*REDACTED*" } ] }"#).unwrap()
}

pub fn make_shadow_vcard() -> Value {
    serde_json::from_str(r#"{ "vcardArray": [ "vcard", [ ["version", {}, "text", "4.0"], ["fn", {}, "text", "*REDACTED*"], ["n", {}, "text", ["User", "*REDACTED*", "*REDACTED*", "*REDACTED*", ["*REDACTED*", "*REDACTED*"]]], ["kind", {}, "text", "individual"], ["lang", { "pref":"1" }, "language-tag", "*REDACTED*"], ["lang", { "pref":"2" }, "language-tag", "*REDACTED*"], ["org", { "type":"work" }, "text", "*REDACTED*"], ["title", {}, "text", "*REDACTED*"], ["role", {}, "text", "*REDACTED*"], ["adr", { "type":"work" }, "text", ["*REDACTED*", "*REDACTED*"]], ["adr", { "type":"home", "label":"*REDACTED*" }, "text", ["*REDACTED*", "*REDACTED*"]], ["tel", { "type":["work", "voice"], "pref":"1" }, "uri", "*REDACTED*"], ["tel", { "type":["*REDACTED*", "*REDACTED*", "*REDACTED*", "*REDACTED*", "*REDACTED*"] }, "uri", "*REDACTED*"], ["email", { "type":"work" }, "text", "*REDACTED*"], ["geo", { "type":"work" }, "uri", "*REDACTED*"], ["key", { "type":"work" }, "uri", "*REDACTED*"], ["tz", {}, "utc-offset", "-05:00"], ["url", { "type":"*REDACTED*" }, "uri", "*REDACTED*"] ] ] }"#).unwrap()
}

pub fn make_shadow_port43() -> Value {
    serde_json::from_str(r#"{ "port43":"*REDACTED*" }"#).unwrap()
}

pub fn make_shadow_status() -> Value {
    serde_json::from_str(r#"{ "status":[ "*REDACTED*", "*REDACTED*" ] }"#).unwrap()
}

pub fn make_shadow_handle() -> Value {
    serde_json::from_str(r#"{ "handle":"*REDACTED*" }"#).unwrap()
}

pub fn make_shadow_roles() -> Value {
    serde_json::from_str(r#"{ "roles": [ "*REDACTED*" ] }"#).unwrap()
}

pub fn make_shadow_ldhName() -> Value {
    serde_json::from_str(r#"{ "ldhName":"*REDACTED*" }"#).unwrap()
}

pub fn make_shadow_unicodeName() -> Value {
    serde_json::from_str(r#"{ "unicodeName":"*REDACTED*" }"#).unwrap()
}

pub fn make_ip_addresses() -> Value {
    serde_json::from_str(r#"{ "ipAddresses": [ { "v4": [ "*REDACTED*" ], "v6": [ "*REDACTED*" ] } ] }"#).unwrap()
}

pub fn makes_secureDNS() -> Value {
    serde_json::from_str(r#"{ "secureDNS": { "zoneSigned": true, "delegationSigned": true, "maxSigLife": null, "dsData": [ { "keyTag": null, "algorithm": null, "digest": "*REDACTED*", "digestType": null } ], "keyData": [ { "flags": null, "protocol": null, "algorithm": null, "publicKey": "*REDACTED*" } ] } }"#).unwrap()
}

pub fn make_shadow_entities() -> Value {
    serde_json::from_str(r#"{ "entities" : [ ] }"#).unwrap()
}
