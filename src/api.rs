//! The definition of the browser API for `semantic_editor`.
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(missing_docs)]

use std::panic;

use wasm_bindgen::prelude::*;

use logger;
use executor;

#[wasm_bindgen]
pub fn init() {
    logger::init();

    panic::set_hook(Box::new(|info| {
        error!("panic occurred: {}", info.payload().downcast_ref::<&str>().unwrap());
    }))
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct SemanticEditor(super::SemanticEditor);

#[wasm_bindgen]
impl SemanticEditor {
    pub fn new(url: &str, resolve: JsValue, reject: JsValue) {
        use futures::Future;

        let executor = executor::Executor::new();

        let future = super::SemanticEditor::new(executor.clone(), url.to_owned())
            .map(move |e| resolveSemanticEditor(resolve, SemanticEditor(e)))
            .map_err(move |e| rejectSemanticEditor(reject, &format!("{}", e)));

        executor.spawn(future);
    }

    pub fn send_rpc(&self) {
        use futures::Future;

        self.0.executor.spawn(self.0.send_rpc().map_err(|e| error!("rpc failed: {:?}", e)));
    }

    pub fn document(&self) -> String {
        self.0.document()
    }
}

#[wasm_bindgen(module = "../ffi")]
extern "C" {
    #[allow(non_snake_case)]
    fn resolveSemanticEditor(resolve: JsValue, semanticEditor: SemanticEditor);
    #[allow(non_snake_case)]
    fn rejectSemanticEditor(reject: JsValue, error: &str);
}
