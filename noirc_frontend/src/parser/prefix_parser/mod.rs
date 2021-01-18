mod array;
mod declaration;
mod function;
mod group;
mod r#if;
mod literal;
mod name;
mod unary;
mod r#use;
mod module;
mod r#for;
mod constrain;
mod path;

use unary::UnaryParser;
use group::GroupParser;
use literal::LiteralParser;
pub(super) use name::NameParser;
use r#for::ForParser;

pub use array::ArrayParser;
pub use declaration::DeclarationParser;
pub use function::FuncParser;
pub use r#if::IfParser;
pub use r#use::UseParser;
pub use module::ModuleParser;
pub use constrain::ConstrainParser;
pub use path::PathParser;

/// This file defines all Prefix parser ie it defines how we parser statements which begin with a specific token or token type
use crate::ast::{
    ArrayLiteral, BlockStatement, Expression, ExpressionKind,FunctionDefinition, Ident,
    IfExpression, ForExpression, Literal, PrefixExpression, Type, NoirFunction
};
use crate::token::{Keyword, Token, TokenKind, Attribute};
use noirc_errors::{Spanned, Span};


use super::{Parser, Precedence, ParserError,ParserExprKindResult,ParserExprResult};
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
}

impl PrefixParser {
    pub fn parse(&self,parser: &mut Parser) -> ParserExprResult {
        match self {
            PrefixParser::For => span_parser(parser,ForParser::parse),
            PrefixParser::If => span_parser(parser,IfParser::parse),
            PrefixParser::Array => span_parser(parser,ArrayParser::parse),
            PrefixParser::Name => span_parser(parser,NameParser::parse),
            PrefixParser::Literal => span_parser(parser,LiteralParser::parse),
            PrefixParser::Unary => span_parser(parser,UnaryParser::parse),
            PrefixParser::Group => span_parser(parser,GroupParser::parse),
            PrefixParser::Path => span_parser(parser,PathParser::parse),
        }
    }

}

fn span_parser(parser : &mut Parser, f : fn(parser: &mut Parser) -> ParserExprKindResult) -> ParserExprResult{
    let start = parser.curr_token.into_span();
    let kind = f(parser)?;
    let end = parser.curr_token.into_span();

    Ok(Expression {
        kind,
        span : start.merge(end)
    })
}