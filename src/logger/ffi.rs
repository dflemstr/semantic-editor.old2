//! FFI for browser APIs needed by `logger`.
#![allow(missing_docs)]
#![allow(unsafe_code)]
#![allow(missing_debug_implementations)]
#![allow(non_camel_case_types)]

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type console;

    #[wasm_bindgen(static = console)]
    pub fn log(s: &str);

    #[wasm_bindgen(static = console)]
    pub fn info(s: &str);

    #[wasm_bindgen(static = console)]
    pub fn warn(s: &str);

    #[wasm_bindgen(static = console)]
    pub fn error(s: &str);
}
