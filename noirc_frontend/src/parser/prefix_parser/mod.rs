mod array;
mod block;
mod constrain;
mod declaration;
mod for_loop;
mod function;
mod group;
mod if_expr;
mod literal;
mod module;
mod name;
mod path;
mod unary;
mod r#use;

use block::BlockParser;
use for_loop::ForParser;
use group::GroupParser;
use literal::LiteralParser;
pub(super) use name::NameParser;
use unary::UnaryParser;

pub use array::ArrayParser;
pub use constrain::ConstrainParser;
pub use declaration::DeclarationParser;
pub use function::FuncParser;
pub use module::ModuleParser;
pub use path::PathParser;
pub use if_expr::IfParser;
pub use r#use::UseParser;

/// This file defines all Prefix parser ie it defines how we parser statements which begin with a specific token or token type
use crate::ast::{
    ArrayLiteral, BlockExpression, Expression, ExpressionKind, ForExpression, FunctionDefinition,
    Ident, IfExpression, Literal, NoirFunction, PrefixExpression, Type,
};
use crate::token::{Attribute, Keyword, Token, TokenKind};
use noirc_errors::{Span, Spanned};

use super::{Parser, ParserError, ParserExprKindResult, ParserExprResult, Precedence};
use crate::parser::errors::ParserErrorKind;

use crate::ast::{
    ConstStatement, ImportStatement, LetStatement, PrivateStatement, PublicStatement, Statement,
};

/// Strictly speaking, this is not needed as we could import choose_prefix_parser
/// and choose based on the token. This is a bit more modularised and cleaner to read however
pub enum PrefixParser {
    For,
    If,
    Group,
    Literal,
    Name,
    Unary,
    Array,
    Path,
    Block,
}

impl PrefixParser {
    pub fn parse(&self, parser: &mut Parser) -> ParserExprResult {
        match self {
            PrefixParser::For => span_parser(parser, ForParser::parse),
            PrefixParser::If => span_parser(parser, IfParser::parse),
            PrefixParser::Array => span_parser(parser, ArrayParser::parse),
            PrefixParser::Name => span_parser(parser, NameParser::parse),
            PrefixParser::Literal => span_parser(parser, LiteralParser::parse),
            PrefixParser::Unary => span_parser(parser, UnaryParser::parse),
            PrefixParser::Group => span_parser(parser, GroupParser::parse),
            PrefixParser::Path => span_parser(parser, PathParser::parse),
            PrefixParser::Block => span_parser(parser, BlockParser::parse),
        }
    }
}

fn span_parser(
    parser: &mut Parser,
    f: fn(parser: &mut Parser) -> ParserExprKindResult,
) -> ParserExprResult {
    let start = parser.curr_token.into_span();
    let kind = f(parser)?;
    let end = parser.curr_token.into_span();

    Ok(Expression {
        kind,
        span: start.merge(end),
    })
}
