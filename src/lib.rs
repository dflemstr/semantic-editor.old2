#![feature(conservative_impl_trait)]
#![feature(proc_macro)]

extern crate bytes;
#[macro_use]
extern crate error_chain;
extern crate futures;
#[macro_use]
extern crate log;
extern crate prost;
#[macro_use]
extern crate prost_derive;
extern crate semantic;
#[macro_use]
extern crate semantic_derive;
extern crate wasm_bindgen;

pub mod browser_log;
pub mod rpc;
pub mod content;

/// The Protobuf-derived schema for interacting with the editor over different RPC mechanisms.
mod schema {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/se.service.rs"));
}

/// Build version information.
mod version {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/version.rs"));
}

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SemanticEditor {}

#[wasm_bindgen]
pub struct Action {}

#[wasm_bindgen]
impl SemanticEditor {
    pub fn new() -> SemanticEditor {
        SemanticEditor {}
    }

    pub fn init(&self) {
        browser_log::init();
        info!(
            concat!(
                "Initializing ",
                env!("CARGO_PKG_NAME"),
                " version ",
                env!("CARGO_PKG_VERSION"),
                "-{} created {} built {} running on {}"
            ),
            version::short_sha(),
            version::commit_date(),
            version::now(),
            version::target()
        );
    }

    pub fn perform(_action: Action) {}

    pub fn document(&self) -> String {
        r#"{
    "nodes": [
      {
        "object": "block",
        "type": "paragraph",
        "nodes": [
          {
            "object": "text",
            "leaves": [
              {
                "text": "A line of text in a paragraph."
              }
            ]
          }
        ]
      }
    ]
  }"#.to_owned()
    }
}
