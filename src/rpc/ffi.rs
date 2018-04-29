//! FFI for browser APIs needed by `rpc`.
pub mod uuid {
    //! FFI for browser APIs for generating UUIDs.
    #![allow(trivial_casts)]
    #![allow(unsafe_code)]
    #![allow(missing_docs)]
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "uuid", version = "3.2.1")]
    extern "C" {
        /// Generate a UUIDv4 as a string.
        pub fn v4() -> String;
    }
}
