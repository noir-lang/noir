use std::fmt;

use crate::token::TokenKind;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParserLabel {
    Atom,
    BinaryOperator,
    Cast,
    Expression,
    FieldAccess,
    Global,
    IntegerType,
    Parameter,
    Pattern,
    Statement,
    Term,
    TypeExpression,
    TokenKind(TokenKind),
}

impl fmt::Display for ParserLabel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserLabel::Atom => write!(f, "atom"),
            ParserLabel::BinaryOperator => write!(f, "binary operator"),
            ParserLabel::Cast => write!(f, "cast"),
            ParserLabel::Expression => write!(f, "expression"),
            ParserLabel::FieldAccess => write!(f, "field access"),
            ParserLabel::Global => write!(f, "global"),
            ParserLabel::IntegerType => write!(f, "integer type"),
            ParserLabel::Parameter => write!(f, "parameter"),
            ParserLabel::Pattern => write!(f, "pattern"),
            ParserLabel::Statement => write!(f, "statement"),
            ParserLabel::Term => write!(f, "term"),
            ParserLabel::TypeExpression => write!(f, "type expression"),
            ParserLabel::TokenKind(token_kind) => write!(f, "{:?}", token_kind),
        }
    }
}
