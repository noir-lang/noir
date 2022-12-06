pub mod ast;
pub mod graph;
pub mod lexer;
pub mod monomorphisation;
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
