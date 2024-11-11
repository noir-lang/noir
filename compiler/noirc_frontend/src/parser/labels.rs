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
    Function,
    GenericParameter,
    Global,
    Identifier,
    Integer,
    IntegerType,
    Item,
    LValue,
    Parameter,
    Path,
    Pattern,
    Statement,
    Term,
    TraitBound,
    TraitImplItem,
    TraitItem,
    Type,
    TypeExpression,
    TypeOrTypeExpression,
    TokenKind(TokenKind),
    UseSegment,
}

impl fmt::Display for ParsingRuleLabel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParsingRuleLabel::Atom => write!(f, "atom"),
            ParsingRuleLabel::BinaryOperator => write!(f, "binary operator"),
            ParsingRuleLabel::Cast => write!(f, "cast"),
            ParsingRuleLabel::Expression => write!(f, "expression"),
            ParsingRuleLabel::FieldAccess => write!(f, "field access"),
            ParsingRuleLabel::Function => write!(f, "function"),
            ParsingRuleLabel::GenericParameter => write!(f, "generic parameter"),
            ParsingRuleLabel::Global => write!(f, "global"),
            ParsingRuleLabel::Identifier => write!(f, "identifier"),
            ParsingRuleLabel::Integer => write!(f, "integer"),
            ParsingRuleLabel::IntegerType => write!(f, "integer type"),
            ParsingRuleLabel::Item => write!(f, "item"),
            ParsingRuleLabel::LValue => write!(f, "left-hand side of assignment"),
            ParsingRuleLabel::Parameter => write!(f, "parameter"),
            ParsingRuleLabel::Path => write!(f, "path"),
            ParsingRuleLabel::Pattern => write!(f, "pattern"),
            ParsingRuleLabel::Statement => write!(f, "statement"),
            ParsingRuleLabel::Term => write!(f, "term"),
            ParsingRuleLabel::TraitBound => write!(f, "trait bound"),
            ParsingRuleLabel::TraitImplItem => write!(f, "trait impl item"),
            ParsingRuleLabel::TraitItem => write!(f, "trait item"),
            ParsingRuleLabel::Type => write!(f, "type"),
            ParsingRuleLabel::TypeExpression => write!(f, "type expression"),
            ParsingRuleLabel::TypeOrTypeExpression => write!(f, "type or type expression"),
            ParsingRuleLabel::TokenKind(token_kind) => write!(f, "{token_kind}"),
            ParsingRuleLabel::UseSegment => write!(f, "identifier, `crate`, `dep` or `super`"),
        }
    }
}
