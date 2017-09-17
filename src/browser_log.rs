#![allow(non_camel_case_types)]

use log;
use wasm_bindgen::prelude::*;

struct BrowserLog;

pub fn init() {
    log::set_max_level(log::LevelFilter::Trace);
    log::set_logger(&BrowserLog).unwrap();
}

impl log::Log for BrowserLog {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let msg = record
            .module_path()
            .map(|path| format!("{}: {}", path, record.args()))
            .unwrap_or_else(|| format!("{}", record.args()));
        match record.level() {
            log::Level::Error => console::error(&msg),
            log::Level::Warn => console::warn(&msg),
            log::Level::Info => console::info(&msg),
            log::Level::Debug | log::Level::Trace => console::log(&msg),
        }
    }

    fn flush(&self) {}
}

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
