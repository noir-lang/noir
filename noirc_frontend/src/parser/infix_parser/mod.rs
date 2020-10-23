mod binary;
mod call;
mod index;
mod path;

pub use binary::BinaryParser;
pub use call::CallParser;
pub use index::IndexParser;
pub use path::PathParser;

use super::Precedence;
use crate::ast::BinaryOp;
use crate::ast::{CallExpression, CastExpression, Expression, IndexExpression, InfixExpression, NoirPath};
use crate::token::Token;

use super::Parser;
use super::parser::ParserExprResult;


pub enum InfixParser{
    Binary, 
    Call,
    Index,
    Path,
}

impl InfixParser {
    pub fn parse(&self, parser: &mut Parser, left: Expression) -> ParserExprResult {
        match self {
            InfixParser::Binary => BinaryParser::parse(parser, left),
            InfixParser::Call => CallParser::parse(parser, left),
            InfixParser::Index => IndexParser::parse(parser, left),
            InfixParser::Path => PathParser::parse(parser, left),
        }
    }
}