//! FFI for browser APIs needed by `rpc::websocket`.
#![allow(missing_docs)]
#![allow(unsafe_code)]
#![allow(missing_debug_implementations)]
#![allow(non_camel_case_types)]
#![allow(trivial_casts)]

use std::fmt;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct WebSocketHandler(pub super::WebSocketHandler);

#[wasm_bindgen]
impl WebSocketHandler {
    pub fn onclose(&self, code: u16, reason: &str, was_clean: bool) {
        self.0.onclose(code, reason, was_clean)
    }

    pub fn onerror(&self) {
        self.0.onerror()
    }

    pub fn onmessage(&self, data: &[u8], origin: &str) {
        self.0.onmessage(data, origin)
    }

    pub fn onopen(&self) {
        self.0.onopen()
    }
}

impl fmt::Debug for WebSocket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WebSocket(_)")
    }
}

#[wasm_bindgen(module = "../rpc/websocket/ffi")]
extern "C" {
    #[allow(non_snake_case)]
    pub fn setWebSocketHandler(websocket: &WebSocket, handler: WebSocketHandler);
}

pub mod uuid {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "uuid")]
    extern "C" {
        pub fn v4() -> String;
    }
}

#[wasm_bindgen]
extern "C" {
    pub type WebSocket;

    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> WebSocket;

    #[wasm_bindgen(method)]
    pub fn close(this: &WebSocket, code: u32, reason: &str);

    #[wasm_bindgen(method)]
    pub fn send(this: &WebSocket, data: &[u8]);
}
