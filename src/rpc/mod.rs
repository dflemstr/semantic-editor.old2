//! Common RPC definitions for various communication protocols.
//!
//! These RPC definitions can be used for a wide variety of transport protocols as long as they can
//! agree on using protobuf-derived message schemata.

pub mod websocket;

// TODO: negotiate different transport mechanisms and put an abstraction here.
