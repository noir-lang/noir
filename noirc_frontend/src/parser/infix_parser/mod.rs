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
use crate::ast::{CallExpression, CastExpression, Expression, ExpressionKind, IndexExpression, InfixExpression, NoirPath};
use crate::token::Token;

use super::Parser;
use super::ParserError;
use super::parser::{ParserExprResult, ParserExprKindResult};

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
            InfixParser::Binary => span_parser(parser,left,BinaryParser::parse),
            InfixParser::Call => span_parser(parser,left,CallParser::parse),
            InfixParser::Index => span_parser(parser,left,IndexParser::parse),
            InfixParser::Path => span_parser(parser,left,PathParser::parse),
        }
    }
}

fn span_parser(parser : &mut Parser, left: Expression, f : fn(parser: &mut Parser,left: Expression) -> ParserExprKindResult) -> ParserExprResult{
    let start = parser.curr_token.into_span();
    let kind = f(parser, left)?;
    let end = parser.curr_token.into_span();

    Ok(Expression {
        kind,
        span : start.merge(end)
    })
}