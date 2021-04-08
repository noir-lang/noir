mod errors;
mod infix_parser;
#[allow(clippy::module_inception)]
mod parser;
mod prefix_parser;

use crate::{ast::ImportStatement, NoirFunction};
use crate::{
    token::{Keyword, SpannedToken, Token},
    Ident,
};
pub use errors::ParserErrorKind;
pub use parser::{Parser, ParserExprKindResult, ParserExprResult};

#[derive(Clone, Debug)]
pub struct ParsedModule {
    pub imports: Vec<ImportStatement>,
    pub functions: Vec<NoirFunction>,
    pub module_decls: Vec<Ident>,
}

impl ParsedModule {
    fn with_capacity(cap: usize) -> Self {
        ParsedModule {
            imports: Vec::with_capacity(cap),
            functions: Vec::with_capacity(cap),
            module_decls: Vec::new(),
        }
    }

    fn push_function(&mut self, func: NoirFunction) {
        self.functions.push(func);
    }
    fn push_import(&mut self, import_stmt: ImportStatement) {
        self.imports.push(import_stmt);
    }
    fn push_module_decl(&mut self, mod_name: Ident) {
        self.module_decls.push(mod_name);
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
pub(crate) fn test_parse(src: &str) -> Parser {
    Parser::from_src(src)
}

#[cfg(test)]
pub(crate) fn dummy_expr() -> crate::Expression {
    use crate::parser::prefix_parser::PrefixParser;
    const SRC: &str = r#"
        foo;
    "#;
    let mut parser = test_parse(SRC);
    PrefixParser::Path.parse(&mut parser).unwrap()
}
