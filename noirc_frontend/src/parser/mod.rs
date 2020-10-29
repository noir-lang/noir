mod infix_parser;
mod prefix_parser;
mod parser;

mod errors;

pub use errors::ParserError;

pub use parser::{Parser, ParserExprResult,ParserExprKindResult};

use crate::ast::{Expression, FunctionDefinition, ImportStatement, Statement};
use crate::token::{Keyword, Token, SpannedToken};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Program {
    pub imports: Vec<ImportStatement>,
    pub statements: Vec<Statement>,
    pub functions: Vec<FunctionDefinition>,
    pub main: Option<FunctionDefinition>,
    pub modules : HashMap<String, Program>,
}

const MAIN_FUNCTION: &str = "main";

impl Program {
    pub fn new() -> Self {
        Program::with_capacity(0)
    }
    pub fn with_capacity(cap: usize) -> Self {
        Program {
            imports: Vec::with_capacity(cap),
            statements: Vec::with_capacity(cap),
            functions: Vec::with_capacity(cap),
            main: None,
            modules : HashMap::new(),
        }
    }
    pub fn push_statement(&mut self, stmt: Statement) {
        self.statements.push(stmt)
    }
    pub fn push_function(&mut self, func: FunctionDefinition) {
        if &func.name.0.contents == MAIN_FUNCTION {
            self.main = Some(func)
        } else {
            self.functions.push(func);
        }
    }
    pub fn push_import(&mut self, import_stmt: ImportStatement) {
        self.imports.push(import_stmt);
    }
    pub fn push_module(&mut self, mod_name: String, module : Program) {
        self.modules.insert(mod_name, module);
    }
    /// Returns the program abi which is only present for executables and not libraries
    pub fn abi(&self) -> Option<Vec<(String, crate::ast::Type)>> {
        match &self.main {
            Some(main_func) => {
                let abi = main_func
                    .parameters
                    .iter()
                    .map(|(ident, typ)| (ident.0.contents.clone(), typ.clone()))
                    .collect();
                Some(abi)
            }
            None => None,
        }
    }
}

#[derive(PartialEq, PartialOrd)]
pub enum Precedence {
    Lowest,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    Call,
    Index,
}
impl Precedence {
    // Higher the number, the higher(more priority) the precedence
    // XXX: Check the precedence is correct for operators
    fn token_precedence(tok: &Token) -> Precedence {
        match tok {
            Token::Assign => Precedence::Equals,
            Token::Equal => Precedence::Equals,
            Token::NotEqual => Precedence::Equals,
            Token::Less => Precedence::LessGreater,
            Token::LessEqual => Precedence::LessGreater,
            Token::Greater => Precedence::LessGreater,
            Token::GreaterEqual => Precedence::LessGreater,
            Token::Ampersand => Precedence::Sum,
            Token::Caret => Precedence::Sum,
            Token::Pipe => Precedence::Sum,
            Token::Plus => Precedence::Sum,
            Token::Minus => Precedence::Sum,
            Token::Slash => Precedence::Product,
            Token::Star => Precedence::Product,
            Token::Keyword(Keyword::As) => Precedence::Prefix,
            Token::LeftParen => Precedence::Call,
            Token::LeftBracket => Precedence::Index,
            Token::DoubleColon => Precedence::Index,
            _ => Precedence::Lowest,
        }
    }
}
impl From<&Token> for Precedence {
    fn from(t: &Token) -> Precedence {
        Precedence::token_precedence(t)
    }
}
impl From<&SpannedToken> for Precedence {
    fn from(t: &SpannedToken) -> Precedence {
        Precedence::token_precedence(t.token())
    }
}
