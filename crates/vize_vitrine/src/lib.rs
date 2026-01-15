//! NAPI and WASM bindings for Vue compiler.

#[cfg(feature = "napi")]
pub mod napi;

#[cfg(feature = "wasm")]
pub mod wasm;

pub mod typecheck;
pub mod types;

pub use typecheck::{
    type_check_sfc, RelatedLocation, TypeCheckOptions, TypeCheckResult, TypeDiagnostic,
    TypeSeverity,
};
pub use types::*;
