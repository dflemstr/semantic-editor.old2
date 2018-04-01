//! Target-specific implementation of executor functionality.
#[cfg(not(target_arch = "wasm32"))]
pub mod default;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(target_arch = "wasm32")]
pub use self::wasm::run;
#[cfg(target_arch = "wasm32")]
pub use self::wasm::spawn;

#[cfg(not(target_arch = "wasm32"))]
pub use self::default::run;
#[cfg(not(target_arch = "wasm32"))]
pub use self::default::spawn;
