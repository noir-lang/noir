mod binary;
mod call;
mod index;
mod path;

pub use binary::BinaryParser;
pub use call::CallParser;
pub use index::IndexParser;
pub use path::PathParser;

use crate::Precedence;
use libnoirc_ast::BinaryOp;
use libnoirc_ast::{CallExpression, CastExpression, Expression, IndexExpression, InfixExpression, NoirPath};
use libnoirc_lexer::token::Token;

use crate::{InfixParser, Parser};
