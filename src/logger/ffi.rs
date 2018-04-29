//! FFI for browser APIs needed by `logger`.
#![allow(missing_docs)]
#![allow(unsafe_code)]
#![allow(missing_debug_implementations)]
#![allow(non_camel_case_types)]
#![allow(trivial_numeric_casts)]

use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "./../logger/ffi")]
extern "C" {
    pub fn newObject() -> JsValue;
    pub fn emitUsize(obj: &JsValue, key: &str, val: usize);
    pub fn emitIsize(obj: &JsValue, key: &str, val: isize);
    pub fn emitBool(obj: &JsValue, key: &str, val: bool);
    pub fn emitChar(obj: &JsValue, key: &str, val: u32);
    pub fn emitU8(obj: &JsValue, key: &str, val: u8);
    pub fn emitI8(obj: &JsValue, key: &str, val: i8);
    pub fn emitU16(obj: &JsValue, key: &str, val: u16);
    pub fn emitI16(obj: &JsValue, key: &str, val: i16);
    pub fn emitU32(obj: &JsValue, key: &str, val: u32);
    pub fn emitI32(obj: &JsValue, key: &str, val: i32);
    pub fn emitF32(obj: &JsValue, key: &str, val: f32);
    pub fn emitU64(obj: &JsValue, key: &str, val: u64);
    pub fn emitI64(obj: &JsValue, key: &str, val1: u32, val2: u32);
    pub fn emitF64(obj: &JsValue, key: &str, val: f64);
    pub fn emitStr(obj: &JsValue, key: &str, val: &str);
    pub fn emitUnit(obj: &JsValue, key: &str);
    pub fn emitNone(obj: &JsValue, key: &str);
}

#[wasm_bindgen]
extern "C" {
    pub type console;

    #[wasm_bindgen(js_namespace = console)]
    pub fn log(format: &str, css1: &str, module: &str, css2: &str, msg: &str, kv: JsValue);

    #[wasm_bindgen(js_namespace = console)]
    pub fn info(format: &str, css1: &str, module: &str, css2: &str, msg: &str, kv: JsValue);

    #[wasm_bindgen(js_namespace = console)]
    pub fn warn(format: &str, css1: &str, module: &str, css2: &str, msg: &str, kv: JsValue);

    #[wasm_bindgen(js_namespace = console)]
    pub fn error(format: &str, css1: &str, module: &str, css2: &str, msg: &str, kv: JsValue);
}
