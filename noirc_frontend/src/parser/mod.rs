mod infix_parser;
mod prefix_parser;
mod parser;

mod errors;

pub use errors::ParserError;

pub use parser::{Parser, ParserExprResult,ParserExprKindResult};

use crate::{Expression, FunctionKind, NoirFunction, ast::{FunctionDefinition, ImportStatement, Type}};
use crate::token::{Keyword, Token, SpannedToken};

#[derive(Clone, Debug)]
pub struct Program {
    pub file_id : usize,
    pub imports: Vec<ImportStatement>,
    pub functions: Vec<NoirFunction>,
    pub module_decls : Vec<String>,
}

const MAIN_FUNCTION: &str = "main";

impl Program {
    /// Returns the program abi which is only present for executables and not libraries
    /// Note: That a library can have a main method, so you should only call this, if you are sure the crate is a binary
    pub fn abi(&self) -> Option<Vec<(String, Type)>> {
        let main_func = self.find_function(MAIN_FUNCTION)?;
        match main_func.kind {
            FunctionKind::Normal => Some(Program::func_to_abi(main_func.def())), // The main function should be normal and not a builtin/low level
            _=> None 
        }
        
    }

    pub fn find_function(&self, name : &str) -> Option<&NoirFunction> {
        for func in self.functions.iter() {
            let func_name  = func.name();
            if func_name == name  {
                return Some(func)
            }
        }
        None
    }
        
    fn with_capacity(cap: usize, file_id : usize) -> Self {
        Program {
            file_id,
            imports: Vec::with_capacity(cap),
            functions: Vec::with_capacity(cap),
            module_decls : Vec::new(),
        }
    }

    fn push_function(&mut self, func: NoirFunction) {
        self.functions.push(func);
    }
    fn push_import(&mut self, import_stmt: ImportStatement) {
        self.imports.push(import_stmt);
    }
    fn push_module_decl(&mut self, mod_name: String) {
        self.module_decls.push(mod_name);
    }

    fn func_to_abi(func : &FunctionDefinition) -> Vec<(String, Type)> {
        func
        .parameters
        .iter()
        .map(|(ident, typ)| (ident.0.contents.clone(), typ.clone()))
        .collect()
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

#[cfg(test)]
pub(crate) fn test_parse(src : &str) -> Parser {
    Parser::from_src(Default::default(), src)
}
