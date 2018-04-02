//! FFI for browser APIs needed by `rpc::http`.
#![allow(missing_docs)]
#![allow(unsafe_code)]
#![allow(missing_debug_implementations)]
#![allow(non_camel_case_types)]
#![allow(trivial_casts)]
#![allow(trivial_numeric_casts)]

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug)]
pub struct HttpFetchHandler(pub super::HttpFetchHandler);

#[wasm_bindgen]
impl HttpFetchHandler {
    pub fn resolve(&self, data: &[u8]) {
        self.0.resolve(data)
    }

    pub fn reject(&self, error: &str) {
        self.0.reject(error)
    }
}

#[wasm_bindgen(module = "../rpc/http/ffi")]
extern "C" {
    #[allow(non_snake_case)]
    pub fn performFetch(url: &str, data: &[u8], handler: HttpFetchHandler);
}
