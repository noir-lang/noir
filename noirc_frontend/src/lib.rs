pub mod lexer;
pub mod parser;
pub mod ast;
pub mod symbol_table;

// XXX: I think this API can be cleaned up even more

// Lexer API
pub use lexer::token;

//Parser API
pub use parser::{Parser, Program};

//AST API
pub use ast::*;

// Symbol table
pub use symbol_table::*;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
