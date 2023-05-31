use std::fmt;

use crate::token::TokenKind;

/// Used to annotate parsing rules with extra context that can be presented to the user later in
/// the case of an error.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsingRuleLabel {
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

impl fmt::Display for ParsingRuleLabel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParsingRuleLabel::Atom => write!(f, "atom"),
            ParsingRuleLabel::BinaryOperator => write!(f, "binary operator"),
            ParsingRuleLabel::Cast => write!(f, "cast"),
            ParsingRuleLabel::Expression => write!(f, "expression"),
            ParsingRuleLabel::FieldAccess => write!(f, "field access"),
            ParsingRuleLabel::Global => write!(f, "global"),
            ParsingRuleLabel::IntegerType => write!(f, "integer type"),
            ParsingRuleLabel::Parameter => write!(f, "parameter"),
            ParsingRuleLabel::Pattern => write!(f, "pattern"),
            ParsingRuleLabel::Statement => write!(f, "statement"),
            ParsingRuleLabel::Term => write!(f, "term"),
            ParsingRuleLabel::TypeExpression => write!(f, "type expression"),
            ParsingRuleLabel::TokenKind(token_kind) => write!(f, "{:?}", token_kind),
        }
    }
}
