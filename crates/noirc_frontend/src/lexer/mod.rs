#[allow(clippy::module_inception)]
pub mod lexer;
pub mod token;
pub use lexer::{Lexer, SpannedTokenResult};

pub mod errors;
