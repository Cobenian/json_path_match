
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
use serde_json::Value;
use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize)]
pub struct ShadowBuilder {
  pub conformances: Value,
  pub links: Value,
  pub notices_and_remarks:Value,
  pub lang: Value,
  pub events: Value,
  pub status: Value,
  pub port43: Value,
  pub public_ids: Value,
  pub object_class_name: Value
}

// use serde::ser::{Serialize, Serializer, SerializeStruct};

// impl Serialize for ShadowDomainBuilder {
//   fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//   where
//       S: Serializer,
//   {
//       let mut s = serializer.serialize_struct("ShadowDomainBuilder", 17)?;
//       s.serialize_field("domain", &self.domain)?;
//       s.serialize_field("handle", &serde_json::from_str::<serde_json::Value>(&self.handle).unwrap())?;
//       s.serialize_field("ldhName", &serde_json::from_str::<serde_json::Value>(&self.ldhName).unwrap())?;
//       s.serialize_field("unicodeName", &serde_json::from_str::<serde_json::Value>(&self.unicodeName).unwrap())?;
//       s.serialize_field("status", &self.status)?;
//       s.serialize_field("roles", &self.roles)?;
//       s.serialize_field("port43", &serde_json::from_str::<serde_json::Value>(&self.port43).unwrap())?;
//       s.serialize_field("vcard", &serde_json::from_str::<serde_json::Value>(&self.vcard).unwrap())?;
//       s.serialize_field("publicIds", &serde_json::from_str::<serde_json::Value>(&self.publicIds).unwrap())?;
//       s.serialize_field("events", &serde_json::from_str::<serde_json::Value>(&self.events).unwrap())?;
//       s.serialize_field("lang", &serde_json::from_str::<serde_json::Value>(&self.lang).unwrap())?;
//       s.serialize_field("remarks", &serde_json::from_str::<serde_json::Value>(&self.remarks).unwrap())?;
//       s.serialize_field("notices", &serde_json::from_str::<serde_json::Value>(&self.notices).unwrap())?;
//       s.serialize_field("links", &serde_json::from_str::<serde_json::Value>(&self.links).unwrap())?;
//       s.serialize_field("entities", &serde_json::from_str::<serde_json::Value>(&self.entities).unwrap())?;
//       s.serialize_field("ipAddresses", &serde_json::from_str::<serde_json::Value>(&self.ipAddresses).unwrap())?;
//       s.serialize_field("secureDNS", &serde_json::from_str::<serde_json::Value>(&self.secureDNS).unwrap())?;
//       s.end()
//   }
// }

// // #[derive(serde::Serialize)] 
// // pub struct ShadowDomainBuilder {
// //   domain: String,
// //   handle: serde_json::Value,
// //   ldhName: serde_json::Value,
// //   unicodeName: serde_json::Value,
// //   status: serde_json::Value,
// //   roles: serde_json::Value,
// //   port43: serde_json::Value,
// //   vcard: serde_json::Value,
// //   publicIds: serde_json::Value,
// //   events: serde_json::Value,
// //   lang: serde_json::Value,
// //   remarks: serde_json::Value,
// //   notices: serde_json::Value,
// //   links: serde_json::Value,
// //   entities: serde_json::Value,
// //   ipAddresses: serde_json::Value,
// //   secureDNS: serde_json::Value,
// // }

// impl ShadowDomainBuilder {
//   pub fn new() -> Self {
//       ShadowDomainBuilder {
//           domain: String::new(),
//           handle: make_shadow_redacted_string(),
//           ldhName: make_shadow_redacted_string(),
//           unicodeName: make_shadow_redacted_string(),
//           status: make_shadow_status(),
//           roles: make_shadow_roles(),
//           port43: make_shadow_port43(),
//           vcard: make_shadow_vcard(),
//           publicIds: make_shadow_publicIds(),
//           events: make_shadow_events(),
//           lang: make_shadow_lang(),
//           remarks: make_shadow_remarks(),
//           notices: make_shadow_notices(),
//           links: make_shadow_links(),
//           entities: make_shadow_entities(),
//           ipAddresses: make_ip_addresses(),
//           secureDNS: makes_secureDNS()
//           // Initialize other fields...
//       }
//   }


//   pub fn domain(mut self, domain: String) -> Self {
//       self.domain = domain;
//       self
//   }

//   pub fn build(self) -> Self {
//       self
//   }
// }


fn make_shadow_nameserver() {

}

fn make_shadow_entity() {

}

fn make_shadow_ip_network() {

}

fn make_shadow_asn() {

}

fn make_shadow_redacted_string() -> String {
  return "*REDACTED*".to_string();
}


fn make_shadow_redacted_array() -> Vec<String> {
  return vec!["*REDACTED*".to_string()];
}
// LINKS
// Example
//  {
//   "value" : "https://example.com/context_uri",
//   "rel" : "self",
//   "href" : "https://example.com/target_uri",
//   "hreflang" : [ "en", "ch" ],
//   "title" : "title",
//   "media" : "screen",
//   "type" : "application/json"
// }
// BUILDS a Shadow Link
// {
//   "value" : "*REDACTED*",
//   "rel" : "*REDACTED*",
//   "href" : "*REDACTED*",
//   "hreflang" : [ "*REDACTED*"],
//   "title" : "*REDACTED*",
//   "media" : "*REDACTED*",
//   "type" : "*REDACTED*"
// }
// pub fn make_shadow_links() -> String {
//   let json_str = r#"
//   [
//     {
//       value : *REDACTED*,
//       rel : *REDACTED*,
//       href : *REDACTED*,
//       hreflang : [ *REDACTED*],
//       title : *REDACTED*,
//       media : *REDACTED*,
//       type : *REDACTED*
//     }
//   ]
//   "#.split_whitespace().collect::<Vec<&str>>().join(" ");
//   json_str
// }

pub fn make_shadow_links() -> Value {  // Change the return type here
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
  serde_json::from_str(json_str).unwrap()  // Parse the JSON string into a Value
}

fn make_shadow_notices() -> String {
  return r#"
"notices" :
[
  {
    "title" : "*REDACTED*",
    "description" :
    [
      "*REDACTED*",
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
"#.split_whitespace().collect();
}

fn make_shadow_remarks() -> String {
  return r#"
  "remarks" :
[
  "type" : "*REDACTED*",
  {
    "description" :
    [
      "*REDACTED*"
    ]
  }
]
"#.split_whitespace().collect();
}

fn make_shadow_lang() -> String {
  return r#" "lang" : "*REDACTED*" "#.split_whitespace().collect();
}

fn make_shadow_events() -> String {
  return r#"
"events" :
[
  {
    "eventAction" : "*REDACTED*",
    "eventActor" : "*REDACTED*",
    "eventDate" : "*REDACTED*"
  }
]
"#.split_whitespace().collect();
}

fn make_shadow_publicIds() -> String {
  return r#"
  "publicIds":
[
  {
    "type":"*REDACTED*",
    "identifier":"*REDACTED*"
  }
]
"#.split_whitespace().collect();
}

fn make_shadow_vcard() -> String {
  return r#"
  vcardArray":[
    "vcard",
    [
      ["version", {}, "text", "4.0"],
      ["fn", {}, "text", "*REDACTED*"],
      ["n", {}, "text",
        ["User", "*REDACTED*", "*REDACTED*", "*REDACTED*", ["*REDACTED*", "*REDACTED*"]]
      ],
      ["kind", {}, "text", "individual"],
      ["lang", {
        "pref":"1"
      }, "language-tag", "*REDACTED*"],
      ["lang", {
        "pref":"2"
      }, "language-tag", "*REDACTED*],
      ["org", {
        "type":"work"
      }, "text", "*REDACTED*"],
      ["title", {}, "text", "*REDACTED*"],
      ["role", {}, "text", "*REDACTED*"],
      ["adr",
        { "type":"work" },
        "text",
        [
          "*REDACTED*",
          "*REDACTED*"
        ]
      ],
      ["adr",
        {
          "type":"home",
          "label":"*REDACTED*"
        },
        "text",
        [
          "*REDACTED*", "*REDACTED*"
        ]
      ],
      ["tel",
        {
          "type":["work", "voice"],
          "pref":"1"
        },
        "uri",
        "*REDACTED*"
      ],
      ["tel",
        { "type":["*REDACTED*", "*REDACTED*", "*REDACTED*", "*REDACTED*", "*REDACTED*"] },
        "uri",
        "*REDACTED*"
      ],
      ["email",
        { "type":"work" },
        "text",
        "*REDACTED*"
      ],
      ["geo", {
        "type":"work"
      }, "uri", "*REDACTED*"],
      ["key",
        { "type":"work" },
        "uri",
        "*REDACTED*"
      ],
      ["tz", {},
        "utc-offset", "-05:00"],
      ["url", { "type":"*REDACTED*" },
        "uri", "*REDACTED*"]
    ]
  ]
"#.split_whitespace().collect();
}

fn make_shadow_port43() -> String {
  return r#"
  "port43":"*REDACTED*"
"#.split_whitespace().collect();
}

fn make_shadow_status() -> String {
  return r#"
  "status":[ "*REDACTED*", "*REDACTED*" ]
"#.split_whitespace().collect();
}

fn make_shadow_handle() -> String {
  return r#"
  "*REDACTED*" 
"#.split_whitespace().collect();
}

fn make_shadow_roles() -> String {
  return r#"
  "roles": [ "*REDACTED*" ]
"#.split_whitespace().collect();
}

fn make_shadow_ldhName() -> String {
  return r#"
  "ldhName":"*REDACTED*"  
"#.split_whitespace().collect();
}

fn make_shadow_unicodeName() -> String {
  return r#"
  "unicodeName":"*REDACTED*"  
"#.split_whitespace().collect();
}

fn make_ip_addresses() -> String {
  return r#"
  "ipAddresses":
  [
    {
      "v4": [ "*REDACTED*" ],
      "v6": [ "*REDACTED*" ]
    }
  ] 
"#.split_whitespace().collect();
}

fn makes_secureDNS() -> String {
  return r#"
  "secureDNS":
  {
    "zoneSigned": true,
    "delegationSigned": true,
    "maxSigLife": null,
    "dsData":
    [
      {
        "keyTag": null,
        "algorithm": null,
        "digest": "*REDACTED*",
        "digestType": null
      }
    ],
    "keyData": [
      {
        "flags": null,
        "protocol": null,
        "algorithm": null,
        "publicKey": "*REDACTED*"
      }
    ]
  }
"#.split_whitespace().collect();
}

// XXX what if I removed all the entities objects, what would be here
// to show that it has been redacted? An empty entitiy object of what type?
fn make_shadow_entities() -> String {
  return r#"
  "entities" :
  [
    
  ]
"#.split_whitespace().collect(); 
}


