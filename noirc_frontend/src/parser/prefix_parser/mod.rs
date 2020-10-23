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

/// This file defines all Prefix parser ie it defines how we parser statements which begin with a specific token or token type
use crate::ast::{
    ArrayLiteral, BlockStatement, Expression, FunctionDefinition, Ident,
    IfStatement, ForExpression, Literal, PrefixExpression, Type,
};
use crate::token::{Keyword, Token, TokenKind, Attribute};

use super::{Parser, Precedence,  Program, ParserError,ParserExprResult};

use crate::ast::{
    ConstStatement, ImportStatement, LetStatement, PrivateStatement, PublicStatement, Statement,
};

/// Strictly speaking, this is not needed as we could import choose_prefix_parser
/// and choose based on the token. This is a bit more modularised and cleaner to read however
pub enum PrefixParser {
    For,
    Group,
    Literal,
    Name,
    Unary,
    Array,
}

impl PrefixParser {
    pub fn parse(&self,parser: &mut Parser) -> ParserExprResult {
        match self {
            PrefixParser::For => ForParser::parse(parser),
            PrefixParser::Array => ArrayParser::parse(parser),
            PrefixParser::Name => NameParser::parse(parser),
            PrefixParser::Literal => LiteralParser::parse(parser),
            PrefixParser::Unary => UnaryParser::parse(parser),
            PrefixParser::Group => GroupParser::parse(parser),
        }
    }
}
