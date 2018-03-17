//! The semantic editor is a versatile editor for different kinds of content.
//!
//! It edits content *semantically*.  You don't manipulate characters, but rather the structure of
//! your content.  It is impossible to make syntax errors or break style guides.
//!
//! This program is in an early state of development!

#![feature(conservative_impl_trait)]
#![feature(generators)]
#![feature(proc_macro)]
#![cfg_attr(feature = "lint", feature(plugin))]
#![cfg_attr(feature = "lint", plugin(clippy))]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(missing_copy_implementations)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unsafe_code)]
#![deny(unused_import_braces)]
#![deny(unused_qualifications)]
#![deny(non_camel_case_types)]

extern crate bytes;
#[macro_use]
extern crate error_chain;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate futures_await as futures;
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

pub mod api;
pub mod logger;
pub mod data;
pub mod error;
pub mod schema;
pub mod scheduler;
pub mod rpc;
mod version;

use futures::prelude::*;

/// An instance of the semantic editor.
#[derive(Debug)]
pub struct SemanticEditor {
    scheduler: scheduler::Scheduler,
    rpc: rpc::websocket::WebSocketRpc,
}

impl SemanticEditor {
    /// Creates a new semantic editor with default options depending on the environment.
    #[async]
    pub fn new(
        scheduler: scheduler::Scheduler,
        url: String,
    ) -> Result<SemanticEditor, error::Error> {
        logger::init();
        version::log();
        let rpc = await!(rpc::websocket::WebSocketRpc::open(&url))?;
        Ok(SemanticEditor { scheduler, rpc })
    }

    /// (TEMP) Test document rendering
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
