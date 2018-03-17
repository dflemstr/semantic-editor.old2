//! An implementation of `log::Log` for the browser environment.
//!
//! Any log messages are sent to the browser `console` object.
use log;

pub mod ffi;

struct BrowserLog;

/// Set the global logger to be the browser logger.
///
/// Any log messages are sent to the browser `console` object.
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
            log::Level::Error => ffi::console::error(&msg),
            log::Level::Warn => ffi::console::warn(&msg),
            log::Level::Info => ffi::console::info(&msg),
            log::Level::Debug | log::Level::Trace => ffi::console::log(&msg),
        }
    }

    fn flush(&self) {}
}
