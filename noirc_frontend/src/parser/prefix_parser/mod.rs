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

pub use array::ArrayParser;
pub use declaration::DeclarationParser;
pub use function::FuncParser;
pub use group::GroupParser;
pub use literal::LiteralParser;
pub use name::NameParser;
pub use r#if::IfParser;
pub use r#use::UseParser;
pub use r#for::ForParser;
pub use unary::UnaryParser;
pub use module::ModuleParser;

/// This file defines all Prefix parser ie it defines how we parser statements which begin with a specific token or token type
use crate::ast::{
    ArrayLiteral, BlockStatement, Expression, FunctionDefinition, Ident,
    IfStatement, ForExpression, Literal, PrefixExpression, Type,
};
use crate::token::{Keyword, Token, TokenKind};

use super::{Parser, Precedence, PrefixParser, Program};

use crate::ast::{
    ConstStatement, ImportStatement, LetStatement, PrivateStatement, PublicStatement, Statement,
};
