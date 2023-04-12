//! The noir compiler is separated into the following passes which are listed
//! in order in square brackets. The inputs and outputs of each pass are also given:
//!
//! Source file -[Lexing]-> Tokens -[Parsing]-> Ast -[Name Resolution]-> Hir -[Type Checking]-> Hir -[Monomorphization]-> Monomorphized Ast
//!
//! After the monomorphized ast is created, it is passed to the noirc_evaluator crate to convert it to SSA form,
//! perform optimizations, convert to ACIR and eventually prove/verify the program.
#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

pub mod ast;
pub mod graph;
pub mod lexer;
pub mod monomorphization;
pub mod node_interner;
pub mod parser;

pub mod hir;
pub mod hir_def;

// Lexer API
pub use lexer::token;

// Parser API
pub use parser::{parse_program, ParsedModule};

// AST API
pub use ast::*;

// Type API
pub use hir_def::types::*;
