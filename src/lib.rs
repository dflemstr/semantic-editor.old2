//! The semantic editor is a versatile editor for different kinds of content.
//!
//! It edits content *semantically*.  You don't manipulate characters, but rather the structure of
//! your content.  It is impossible to make syntax errors or break style guides.
//!
//! This program is in an early state of development!

#![feature(generators)]
#![feature(nll)]
#![feature(proc_macro)]
#![feature(wasm_import_module)]
#![feature(wasm_custom_section)]
// Blocked on https://github.com/rust-lang-nursery/rust-clippy/issues/2560
// #![cfg_attr(feature = "lint", feature(plugin))]
// #![cfg_attr(feature = "lint", plugin(clippy))]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(missing_copy_implementations)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unsafe_code)]
#![deny(unused_import_braces)]
#![deny(unused_qualifications)]
#![deny(non_camel_case_types)]

#[cfg(not(target_arch = "wasm32"))]
extern crate brotli_decompressor;
extern crate bytes;
extern crate failure;
extern crate futures_await as futures;
#[cfg(not(target_arch = "wasm32"))]
extern crate hyper;
#[cfg(not(target_arch = "wasm32"))]
#[macro_use]
extern crate lazy_static;
#[cfg(not(target_arch = "wasm32"))]
extern crate mime;
extern crate prost;
#[macro_use]
extern crate prost_derive;
extern crate prost_simple_rpc;
extern crate pulldown_cmark;
extern crate semantic;
#[macro_use]
extern crate semantic_derive;
#[macro_use]
extern crate slog;
#[cfg(not(target_arch = "wasm32"))]
extern crate slog_async;
#[cfg(all(not(target_arch = "wasm32"), feature = "journald"))]
extern crate slog_journald;
extern crate slog_scope;
extern crate slog_stdlog;
#[cfg(all(not(target_arch = "wasm32"), feature = "syslog"))]
extern crate slog_syslog;
#[cfg(not(target_arch = "wasm32"))]
extern crate slog_term;
#[cfg(not(target_arch = "wasm32"))]
#[macro_use]
extern crate structopt;
#[cfg(not(target_arch = "wasm32"))]
extern crate tokio;
extern crate tokio_executor;
#[cfg(not(target_arch = "wasm32"))]
extern crate unicase;
extern crate uuid;
extern crate wasm_bindgen;

#[cfg(not(target_arch = "wasm32"))]
pub mod core;
pub mod data;
pub mod error;
pub mod executor;
pub mod logger;
pub mod schema;
pub mod rpc;
mod version;
#[cfg(target_arch = "wasm32")]
pub mod wasm_api;

pub use schema::se::service::SemanticEditor;
