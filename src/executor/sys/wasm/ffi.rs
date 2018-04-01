//! FFI for browser APIs needed by `scheduler`.
#![allow(missing_docs)]
#![allow(unsafe_code)]
#![allow(trivial_numeric_casts)]
#![allow(missing_debug_implementations)]
#![allow(non_camel_case_types)]

use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "../executor/sys/wasm/ffi")]
extern "C" {
    #[allow(non_snake_case)]
    pub fn scheduleMicrotask(microtask: Microtask);
}

#[wasm_bindgen]
pub struct Microtask(pub super::Microtask);

#[wasm_bindgen]
impl Microtask {
    pub fn run(&self) -> bool {
        self.0.run()
    }
}
