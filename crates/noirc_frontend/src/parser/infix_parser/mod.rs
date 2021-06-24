mod binary;
mod call;
mod cast;
mod index;

use binary::BinaryParser;
use call::CallParser;
use cast::CastParser;
use index::IndexParser;

use super::Precedence;
use crate::ast::{BinaryOp, BinaryOpKind};
use crate::ast::{
    CallExpression, CastExpression, Expression, ExpressionKind, IndexExpression, InfixExpression,
};
use crate::token::Token;

use super::parser::{ParserExprKindResult, ParserExprResult};
use super::Parser;
use super::ParserErrorKind;

/// Strictly speaking, this is not needed as we could import choose_prefix_parser
/// and choose based on the token. This is a bit more modularised and cleaner to read however
pub enum InfixParser {
    Binary,
    Call,
    Index,
    Cast,
}

impl InfixParser {
    pub fn parse(&self, parser: &mut Parser, left: Expression) -> ParserExprResult {
        match self {
            InfixParser::Binary => span_parser(parser, left, BinaryParser::parse),
            InfixParser::Call => span_parser(parser, left, CallParser::parse),
            InfixParser::Index => span_parser(parser, left, IndexParser::parse),
            InfixParser::Cast => span_parser(parser, left, CastParser::parse),
        }
    }
}

fn span_parser(
    parser: &mut Parser,
    left: Expression,
    f: fn(parser: &mut Parser, left: Expression) -> ParserExprKindResult,
) -> ParserExprResult {
    let start = parser.curr_token.to_span();
    let kind = f(parser, left)?;
    let end = parser.curr_token.to_span();

    Ok(Expression {
        kind,
        span: start.merge(end),
    })
}
