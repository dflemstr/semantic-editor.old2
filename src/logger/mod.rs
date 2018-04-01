//! An implementation of `log::Log` for the browser environment.
//!
//! Any log messages are sent to the browser `console` object.
use std::fmt;

use slog;
use wasm_bindgen;

pub mod ffi;

struct BrowserDrain;

struct ObjectSerializer(wasm_bindgen::JsValue);

/// Create the global logger to be the browser logger.
///
/// Any log messages are sent to the browser `console` object.
pub fn init() -> slog::Logger {
    slog::Logger::root(BrowserDrain, o!())
}

const FORMAT: &'static str = "%c%s%c: %s %o";
const CSS1: &'static str = "font-weight:bold;font-size:1.2em;font-family:\"SFMono-Regular\",\
                            Consolas,\"Liberation Mono\",Menlo,Courier,monospace;";
const CSS2: &'static str = "font-weight:normal;font-size:1.2em;font-family:-apple-system,\
BlinkMacSystemFont,\"Segoe UI\",Roboto,Oxygen-Sans,Ubuntu,Cantarell,\"Helvetica Neue\",sans-serif;";

impl slog::Drain for BrowserDrain {
    type Ok = ();
    type Err = slog::Never;

    fn log(
        &self,
        record: &slog::Record,
        values: &slog::OwnedKVList,
    ) -> Result<Self::Ok, Self::Err> {
        let module = record.module();
        let msg = format!("{}", record.msg());

        let mut serializer = ObjectSerializer::new();
        slog::KV::serialize(&values, record, &mut serializer).unwrap();
        slog::KV::serialize(&record.kv(), record, &mut serializer).unwrap();
        let kv = serializer.into_object();

        match record.level() {
            slog::Level::Critical | slog::Level::Error => {
                ffi::console::error(FORMAT, CSS1, module, CSS2, &msg, kv)
            }
            slog::Level::Warning => ffi::console::warn(FORMAT, CSS1, module, CSS2, &msg, kv),
            slog::Level::Info => ffi::console::info(FORMAT, CSS1, module, CSS2, &msg, kv),
            slog::Level::Debug | slog::Level::Trace => {
                ffi::console::log(FORMAT, CSS1, module, CSS2, &msg, kv)
            }
        }
        Ok(())
    }
}

impl ObjectSerializer {
    fn new() -> ObjectSerializer {
        ObjectSerializer(ffi::newObject())
    }

    fn into_object(self) -> wasm_bindgen::JsValue {
        self.0
    }
}

impl slog::Serializer for ObjectSerializer {
    fn emit_arguments(&mut self, key: slog::Key, val: &fmt::Arguments) -> slog::Result {
        ffi::emitStr(&self.0, key, &format!("{}", val));
        Ok(())
    }

    fn emit_usize(&mut self, key: slog::Key, val: usize) -> slog::Result {
        ffi::emitUsize(&self.0, key, val);
        Ok(())
    }

    fn emit_isize(&mut self, key: slog::Key, val: isize) -> slog::Result {
        ffi::emitIsize(&self.0, key, val);
        Ok(())
    }

    fn emit_bool(&mut self, key: slog::Key, val: bool) -> slog::Result {
        ffi::emitBool(&self.0, key, val);
        Ok(())
    }

    fn emit_char(&mut self, key: slog::Key, val: char) -> slog::Result {
        ffi::emitChar(&self.0, key, val as u32);
        Ok(())
    }

    fn emit_u8(&mut self, key: slog::Key, val: u8) -> slog::Result {
        ffi::emitU8(&self.0, key, val);
        Ok(())
    }

    fn emit_i8(&mut self, key: slog::Key, val: i8) -> slog::Result {
        ffi::emitI8(&self.0, key, val);
        Ok(())
    }

    fn emit_u16(&mut self, key: slog::Key, val: u16) -> slog::Result {
        ffi::emitU16(&self.0, key, val);
        Ok(())
    }

    fn emit_i16(&mut self, key: slog::Key, val: i16) -> slog::Result {
        ffi::emitI16(&self.0, key, val);
        Ok(())
    }

    fn emit_u32(&mut self, key: slog::Key, val: u32) -> slog::Result {
        ffi::emitU32(&self.0, key, val);
        Ok(())
    }

    fn emit_i32(&mut self, key: slog::Key, val: i32) -> slog::Result {
        ffi::emitI32(&self.0, key, val);
        Ok(())
    }

    fn emit_f32(&mut self, key: slog::Key, val: f32) -> slog::Result {
        ffi::emitF32(&self.0, key, val);
        Ok(())
    }

    fn emit_u64(&mut self, key: slog::Key, val: u64) -> slog::Result {
        ffi::emitU64(&self.0, key, val);
        Ok(())
    }

    fn emit_i64(&mut self, key: slog::Key, val: i64) -> slog::Result {
        ffi::emitI64(&self.0, key, (val >> 32) as u32, val as u32);
        Ok(())
    }

    fn emit_f64(&mut self, key: slog::Key, val: f64) -> slog::Result {
        ffi::emitF64(&self.0, key, val);
        Ok(())
    }

    fn emit_str(&mut self, key: slog::Key, val: &str) -> slog::Result {
        ffi::emitStr(&self.0, key, val);
        Ok(())
    }

    fn emit_unit(&mut self, key: slog::Key) -> slog::Result {
        ffi::emitUnit(&self.0, key);
        Ok(())
    }

    fn emit_none(&mut self, key: slog::Key) -> slog::Result {
        ffi::emitNone(&self.0, key);
        Ok(())
    }
}
