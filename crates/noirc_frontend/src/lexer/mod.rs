//! The lexer is the first pass of the noir compiler.
//! Its goal is to convert a character stream (one input file) into a stream of tokens.
//! See lexer.rs for the lexer itself, and token.rs for more details on tokens.
#[allow(clippy::module_inception)]
pub mod lexer;
pub mod token;
pub use lexer::{Lexer, SpannedTokenResult};

pub mod errors;
