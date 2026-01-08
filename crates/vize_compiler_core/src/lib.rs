//! Vue compiler core - parsing, AST, transforms, and code generation.

pub mod ast;
pub mod codegen;
pub mod errors;
pub mod options;
pub mod parser;
pub mod runtime_helpers;
#[macro_use]
pub mod test_macros;
pub mod tokenizer;
pub mod transform;
pub mod transforms;

pub use ast::*;
pub use codegen::*;
pub use errors::*;
pub use options::*;
pub use parser::*;
pub use runtime_helpers::*;
pub use transform::*;
pub use transforms::*;

/// Re-export allocator types for convenience
pub use vize_allocator::{Allocator, Box as AllocBox, CloneIn, Vec as AllocVec};
