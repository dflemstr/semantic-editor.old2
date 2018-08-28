//! An implementation of `log::Log` for the browser environment.
//!
//! Any log messages are sent to the browser `console` object.
use std::fmt;

use js_sys;
use slog;
use wasm_bindgen;

pub mod ffi;

struct BrowserDrain;

struct ObjectSerializer(js_sys::Object);

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
        ObjectSerializer(js_sys::Object::new())
    }

    fn into_object(self) -> js_sys::Object {
        self.0
    }
}

impl slog::Serializer for ObjectSerializer {
    fn emit_arguments(&mut self, key: slog::Key, val: &fmt::Arguments) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &format!("{}", val).into());
        Ok(())
    }

    fn emit_usize(&mut self, key: slog::Key, val: usize) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &(val as f64).into());
        Ok(())
    }

    fn emit_isize(&mut self, key: slog::Key, val: isize) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &(val as f64).into());
        Ok(())
    }

    fn emit_bool(&mut self, key: slog::Key, val: bool) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &val.into());
        Ok(())
    }

    fn emit_char(&mut self, key: slog::Key, val: char) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &((val as u32) as f64).into());
        Ok(())
    }

    fn emit_u8(&mut self, key: slog::Key, val: u8) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &val.into());
        Ok(())
    }

    fn emit_i8(&mut self, key: slog::Key, val: i8) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &val.into());
        Ok(())
    }

    fn emit_u16(&mut self, key: slog::Key, val: u16) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &val.into());
        Ok(())
    }

    fn emit_i16(&mut self, key: slog::Key, val: i16) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &val.into());
        Ok(())
    }

    fn emit_u32(&mut self, key: slog::Key, val: u32) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &val.into());
        Ok(())
    }

    fn emit_i32(&mut self, key: slog::Key, val: i32) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &val.into());
        Ok(())
    }

    fn emit_f32(&mut self, key: slog::Key, val: f32) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &val.into());
        Ok(())
    }

    fn emit_u64(&mut self, key: slog::Key, val: u64) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &(val as f64).into());
        Ok(())
    }

    fn emit_i64(&mut self, key: slog::Key, val: i64) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &(val as f64).into());
        Ok(())
    }

    fn emit_f64(&mut self, key: slog::Key, val: f64) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &val.into());
        Ok(())
    }

    fn emit_str(&mut self, key: slog::Key, val: &str) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &val.into());
        Ok(())
    }

    fn emit_unit(&mut self, key: slog::Key) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &wasm_bindgen::JsValue::NULL);
        Ok(())
    }

    fn emit_none(&mut self, key: slog::Key) -> slog::Result {
        js_sys::Reflect::set(self.0.as_ref(), &key.into(), &wasm_bindgen::JsValue::NULL);
        Ok(())
    }
}
