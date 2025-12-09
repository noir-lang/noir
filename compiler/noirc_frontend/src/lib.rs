//! The noir compiler is separated into the following passes which are listed
//! in order in square brackets. The inputs and outputs of each pass are also given:
//!
//! ```verbatim
//! Source file -[Lexing]-> Tokens -[Parsing]-> Ast -[Name Resolution]-> Hir -[Type Checking]-> Hir -[Monomorphization]-> Monomorphized Ast
//! ```
//!
//! After the monomorphized ast is created, it is passed to the noirc_evaluator crate to convert it to SSA form,
//! perform optimizations, convert to ACIR and eventually prove/verify the program.
#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
// Temporary allows.
#![allow(clippy::mutable_key_type, clippy::result_large_err)]

pub mod ast;
pub mod debug;
pub mod elaborator;
pub mod graph;
pub mod lexer;
pub mod locations;
pub mod modules;
pub mod monomorphization;
pub mod node_interner;
pub mod ownership;
pub mod parser;
pub mod resolve_locations;
pub mod shared;
pub mod signed_field;
pub mod usage_tracker;

pub mod hir;
pub mod hir_def;

// Lexer API
pub use lexer::token;

// Parser API
pub use parser::{ParsedModule, parse_program, parse_program_with_dummy_file};

// Type API
pub use hir_def::types::*;

// Unit tests that involve all modules
pub mod tests;
// Utility functions for easily compiling the frontend for tests in other crates
pub mod test_utils;
