#![feature(conservative_impl_trait)]
#![feature(proc_macro)]

extern crate bytes;
#[macro_use]
extern crate error_chain;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate futures;
#[macro_use]
extern crate log;
extern crate prost;
#[macro_use]
extern crate prost_derive;
extern crate pulldown_cmark;
extern crate semantic;
#[macro_use]
extern crate semantic_derive;
extern crate uuid;
extern crate wasm_bindgen;

pub mod browser_log;
pub mod data;
pub mod error;
pub mod rpc;

/// The Protobuf-derived schema for interacting with the editor over different RPC mechanisms.
mod se {
    pub mod action {
        #![allow(dead_code)]
        include!(concat!(env!("OUT_DIR"), "/se.action.rs"));
    }
    pub mod data {
        #![allow(dead_code)]
        include!(concat!(env!("OUT_DIR"), "/se.data.rs"));
    }
    pub mod service {
        include!(concat!(env!("OUT_DIR"), "/se.service.rs"));
    }
    pub mod websocket {
        include!(concat!(env!("OUT_DIR"), "/se.websocket.rs"));
    }
}

/// Build version information.
mod version {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/version.rs"));
}

use std::path;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SemanticEditor {}

#[wasm_bindgen]
pub struct File {
    name: path::PathBuf,
}

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

    pub fn create_websocket_rpc(&self, url: &str) {
        use rpc::Client;
        let mut rpc = rpc::websocket::WebSocketRpc::open(url);
        rpc.call("foo", "bar", vec![1, 2, 3].into());
    }

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
