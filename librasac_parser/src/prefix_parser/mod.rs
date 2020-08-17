mod declaration;
mod function;
mod group;
mod r#if;
mod literal;
mod name;
mod unary;

pub use declaration::DeclarationParser;
pub use function::FuncParser;
pub use group::GroupParser;
pub use literal::LiteralParser;
pub use name::NameParser;
pub use r#if::IfParser;
pub use unary::UnaryParser;

/// This file defines all Prefix parser ie it defines how we parser statements which begin with a specific token or token type
// Move these imports into the files which need them.
use librasac_ast::{
    BlockStatement, Expression, FunctionDefinition, FunctionLiteral, Ident, IfExpression, Literal,
    PrefixExpression, Type,
};
use librasac_lexer::token::{Keyword, Token, TokenKind};

use crate::{Parser, Precedence, PrefixParser};

use librasac_ast::{ConstStatement, LetStatement, PrivateStatement, PublicStatement, Statement};
