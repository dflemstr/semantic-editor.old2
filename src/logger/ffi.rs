//! FFI for browser APIs needed by `logger`.
#![allow(missing_docs)]
#![allow(unsafe_code)]
#![allow(missing_debug_implementations)]
#![allow(non_camel_case_types)]
#![allow(trivial_numeric_casts)]

use js_sys;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type console;

    #[wasm_bindgen(js_namespace = console)]
    pub fn log(format: &str, css1: &str, module: &str, css2: &str, msg: &str, kv: js_sys::Object);

    #[wasm_bindgen(js_namespace = console)]
    pub fn info(format: &str, css1: &str, module: &str, css2: &str, msg: &str, kv: js_sys::Object);

    #[wasm_bindgen(js_namespace = console)]
    pub fn warn(format: &str, css1: &str, module: &str, css2: &str, msg: &str, kv: js_sys::Object);

    #[wasm_bindgen(js_namespace = console)]
    pub fn error(format: &str, css1: &str, module: &str, css2: &str, msg: &str, kv: js_sys::Object);
}
