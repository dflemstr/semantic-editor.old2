//! The definition of the browser API for `semantic_editor`.
#![allow(trivial_casts)]
#![allow(trivial_numeric_casts)]
#![allow(unsafe_code)]
#![allow(missing_docs)]

use std::panic;

use slog_scope;
use slog_stdlog;
use wasm_bindgen::prelude::*;

use executor;
use logger;
use rpc;
use schema::se::service;
use version;

#[wasm_bindgen]
#[derive(Debug)]
pub struct SemanticEditor {
    client: service::SemanticEditorClient<rpc::RpcClient<service::SemanticEditorDescriptor>>,
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct FileListing {
    files: Vec<File>,
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct File {
    path: String,
    is_regular: bool,
    is_directory: bool,
}

#[wasm_bindgen]
impl SemanticEditor {
    pub fn new(url: &str, resolve: JsValue, reject: JsValue) {
        use futures::Future;

        let log = logger::init();
        slog_scope::set_global_logger(log.clone()).cancel_reset();
        slog_stdlog::init().unwrap();

        let log = version::init(log);

        let panic_log = log.clone();
        panic::set_hook(Box::new(move |info| {
            error!(
                panic_log,
                "panic occurred: {}",
                info.payload().downcast_ref::<&str>().unwrap()
            );
        }));

        let future = rpc::RpcClient::new(log, url.to_owned())
            .map(|rpc| service::SemanticEditorClient::new(rpc))
            .map(|client| SemanticEditor { client })
            .map(|semantic_editor| resolveSemanticEditor(resolve, semantic_editor))
            .map_err(|err| rejectSemanticEditor(reject, &err.to_string()));

        executor::run(future);
    }

    pub fn list_files(&self, path: &str, resolve: JsValue, reject: JsValue) {
        use futures::Future;
        use schema::se::service::SemanticEditor;

        let path = path.to_owned();
        let future = self
            .client
            .list_files(service::ListFilesRequest { path })
            .map(move |r| {
                resolveFileListing(
                    resolve,
                    FileListing {
                        files: r
                            .file
                            .into_iter()
                            .map(|f| File {
                                path: f.path.to_owned(),
                                is_regular: match f.kind {
                                    Some(service::list_files_response::file::Kind::Regular(_)) => {
                                        true
                                    }
                                    _ => false,
                                },
                                is_directory: match f.kind {
                                    Some(service::list_files_response::file::Kind::Directory(
                                        _,
                                    )) => true,
                                    _ => false,
                                },
                            }).collect(),
                    },
                )
            }).map_err(move |e| rejectFileListing(reject, &e.to_string()));

        executor::run(future);
    }
}

#[wasm_bindgen]
impl FileListing {
    #[allow(non_snake_case)]
    pub fn fileLength(&self) -> usize {
        self.files.len()
    }

    #[allow(non_snake_case)]
    pub fn file(&self, index: usize) -> File {
        self.files[index].clone()
    }
}

#[wasm_bindgen]
impl File {
    pub fn path(&self) -> String {
        self.path.clone()
    }

    #[allow(non_snake_case)]
    pub fn isRegular(&self) -> bool {
        self.is_regular
    }

    #[allow(non_snake_case)]
    pub fn isDirectory(&self) -> bool {
        self.is_directory
    }
}

#[wasm_bindgen(module = "./../ffi")]
extern "C" {
    #[allow(non_snake_case)]
    fn resolveSemanticEditor(resolve: JsValue, semanticEditor: SemanticEditor);
    #[allow(non_snake_case)]
    fn rejectSemanticEditor(reject: JsValue, error: &str);
    #[allow(non_snake_case)]
    fn resolveFileListing(resolve: JsValue, fileListing: FileListing);
    #[allow(non_snake_case)]
    fn rejectFileListing(reject: JsValue, error: &str);
}
