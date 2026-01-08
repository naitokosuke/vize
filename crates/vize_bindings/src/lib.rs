//! NAPI and WASM bindings for Vue compiler.

#[cfg(feature = "napi")]
pub mod napi;

#[cfg(feature = "wasm")]
pub mod wasm;

pub mod types;

pub use types::*;
