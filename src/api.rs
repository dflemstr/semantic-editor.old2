//! The definition of the browser API for `semantic_editor`.
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(missing_docs)]

use std::path;

use wasm_bindgen::prelude::*;

use logger;
use rpc;
use scheduler;
use version;

#[wasm_bindgen]
#[derive(Debug)]
pub struct SemanticEditor(super::SemanticEditor);

#[wasm_bindgen]
impl SemanticEditor {
    pub fn new(url: &str, resolve: JsValue, reject: JsValue) {
        use futures::Future;

        let scheduler = scheduler::Scheduler::new();

        let future = super::SemanticEditor::new(scheduler.clone(), url.to_owned())
            .map(move |e| resolveSemanticEditor(resolve, SemanticEditor(e)))
            .map_err(move |e| rejectSemanticEditor(reject, &format!("{}", e)));

        scheduler.schedule(future);
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
