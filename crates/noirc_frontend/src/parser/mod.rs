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
use noir_field::FieldElement;
pub use parser::{Parser, ParserExprKindResult, ParserExprResult};

#[derive(Clone, Debug)]
pub struct ParsedModule<F> {
    pub imports: Vec<ImportStatement>,
    pub functions: Vec<NoirFunction<F>>,
    pub module_decls: Vec<Ident>,
}

impl<F: FieldElement> ParsedModule<F> {
    fn with_capacity(cap: usize) -> Self {
        ParsedModule {
            imports: Vec::with_capacity(cap),
            functions: Vec::with_capacity(cap),
            module_decls: Vec::new(),
        }
    }

    fn push_function(&mut self, func: NoirFunction<F>) {
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
    fn token_precedence<F: FieldElement>(tok: &Token<F>) -> Precedence {
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
impl<F: FieldElement> From<&Token<F>> for Precedence {
    fn from(t: &Token<F>) -> Precedence {
        Precedence::token_precedence(t)
    }
}
impl<F: FieldElement> From<&SpannedToken<F>> for Precedence {
    fn from(t: &SpannedToken<F>) -> Precedence {
        Precedence::token_precedence(t.token())
    }
}

#[cfg(test)]
pub(crate) fn test_parse(src: &str) -> Parser<ark_bn254::Fr> {
    Parser::from_src(src)
}

#[cfg(test)]
pub(crate) fn dummy_expr() -> crate::Expression<ark_bn254::Fr> {
    use crate::parser::prefix_parser::PrefixParser;
    const SRC: &str = r#"
        foo;
    "#;
    let mut parser = test_parse(SRC);
    PrefixParser::Path.parse(&mut parser).unwrap()
}
