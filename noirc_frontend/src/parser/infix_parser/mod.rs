mod binary;
mod call;
mod index;
mod path;

use binary::BinaryParser;
use call::CallParser;
use index::IndexParser;
use path::PathParser;

use super::Precedence;
use crate::ast::BinaryOp;
use crate::ast::{CallExpression, CastExpression, Expression, IndexExpression, InfixExpression, NoirPath};
use crate::token::Token;

use super::Parser;
use super::parser::ParserExprResult;

/// Strictly speaking, this is not needed as we could import choose_prefix_parser
/// and choose based on the token. This is a bit more modularised and cleaner to read however
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