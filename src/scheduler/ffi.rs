//! FFI for browser APIs needed by `scheduler`.
#![allow(missing_docs)]
#![allow(unsafe_code)]
#![allow(missing_debug_implementations)]
#![allow(non_camel_case_types)]

use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "../scheduler/ffi")]
extern "C" {
    #[allow(non_snake_case)]
    pub fn scheduleMicrotask(microtask: Microtask);
}

#[wasm_bindgen]
pub struct Microtask(pub super::Microtask);

#[wasm_bindgen]
impl Microtask {
    pub fn run(&self) {
        self.0.run()
    }
}
