mod array;
mod declaration;
mod function;
mod group;
mod r#if;
mod literal;
mod name;
mod unary;
mod r#use;

pub use array::ArrayParser;
pub use declaration::DeclarationParser;
pub use function::FuncParser;
pub use group::GroupParser;
pub use literal::LiteralParser;
pub use name::NameParser;
pub use r#if::IfParser;
pub use r#use::UseParser;
pub use unary::UnaryParser;

/// This file defines all Prefix parser ie it defines how we parser statements which begin with a specific token or token type
use libnoirc_ast::{
    ArrayLiteral, BlockStatement, Expression, FunctionDefinition, FunctionLiteral, Ident,
    IfExpression, Literal, PrefixExpression, Type,
};
use libnoirc_lexer::token::{Keyword, Token, TokenKind};

use crate::{Parser, Precedence, PrefixParser};

use libnoirc_ast::{
    ConstStatement, ImportStatement, LetStatement, PrivateStatement, PublicStatement, Statement,
};
