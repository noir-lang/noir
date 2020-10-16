mod binary;
mod call;
mod index;
mod path;

pub use binary::BinaryParser;
pub use call::CallParser;
pub use index::IndexParser;
pub use path::PathParser;

use super::Precedence;
use crate::ast::BinaryOp;
use crate::ast::{CallExpression, CastExpression, Expression, AssignExpression, IndexExpression, InfixExpression, NoirPath};
use crate::token::Token;

use super::{InfixParser, Parser};
