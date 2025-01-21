use super::expr::HirIdent;
use crate::ast::Ident;
use crate::node_interner::{ExprId, StmtId};
use crate::token::SecondaryAttribute;
use crate::Type;
use fm::FileId;
use noirc_errors::{Location, Span};

/// A HirStatement is the result of performing name resolution on
/// the Statement AST node. Unlike the AST node, any nested nodes
/// are referred to indirectly via ExprId or StmtId, which can be
/// used to retrieve the relevant node via the NodeInterner.
#[derive(Debug, Clone)]
pub enum HirStatement {
    Let(HirLetStatement),
    Constrain(HirConstrainStatement),
    Assign(HirAssignStatement),
    For(HirForStatement),
    Loop(ExprId),
    Break,
    Continue,
    Expression(ExprId),
    Semi(ExprId),
    Comptime(StmtId),
    Error,
}

#[derive(Debug, Clone)]
pub struct HirLetStatement {
    pub pattern: HirPattern,
    pub r#type: Type,
    pub expression: ExprId,
    pub attributes: Vec<SecondaryAttribute>,
    pub comptime: bool,
    pub is_global_let: bool,
}

impl HirLetStatement {
    pub fn new(
        pattern: HirPattern,
        r#type: Type,
        expression: ExprId,
        attributes: Vec<SecondaryAttribute>,
        comptime: bool,
        is_global_let: bool,
    ) -> HirLetStatement {
        Self { pattern, r#type, expression, attributes, comptime, is_global_let }
    }

    pub fn ident(&self) -> HirIdent {
        match &self.pattern {
            HirPattern::Identifier(ident) => ident.clone(),
            _ => panic!("can only fetch hir ident from HirPattern::Identifier"),
        }
    }

    pub fn runs_comptime(&self) -> bool {
        self.comptime || self.is_global_let
    }
}

#[derive(Debug, Clone)]
pub struct HirForStatement {
    pub identifier: HirIdent,
    pub start_range: ExprId,
    pub end_range: ExprId,
    pub block: ExprId,
}

/// Corresponds to `lvalue = expression;` in the source code
#[derive(Debug, Clone)]
pub struct HirAssignStatement {
    pub lvalue: HirLValue,
    pub expression: ExprId,
}

/// Corresponds to `constrain expr;` in the source code.
/// This node also contains the FileId of the file the constrain
/// originates from. This is used later in the SSA pass to issue
/// an error if a constrain is found to be always false.
#[derive(Debug, Clone)]
pub struct HirConstrainStatement(pub ExprId, pub FileId, pub Option<ExprId>);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum HirPattern {
    Identifier(HirIdent),
    Mutable(Box<HirPattern>, Location),
    Tuple(Vec<HirPattern>, Location),
    Struct(Type, Vec<(Ident, HirPattern)>, Location),
}

impl HirPattern {
    pub fn field_count(&self) -> usize {
        match self {
            HirPattern::Identifier(_) => 0,
            HirPattern::Mutable(pattern, _) => pattern.field_count(),
            HirPattern::Tuple(fields, _) => fields.len(),
            HirPattern::Struct(_, fields, _) => fields.len(),
        }
    }

    /// Iterate over the fields of this pattern.
    /// Panics if the type is not a struct or tuple.
    pub fn iter_fields<'a>(&'a self) -> Box<dyn Iterator<Item = (String, &'a HirPattern)> + 'a> {
        match self {
            HirPattern::Struct(_, fields, _) => Box::new(
                fields.iter().map(move |(name, pattern)| (name.0.contents.clone(), pattern)),
            ),
            HirPattern::Tuple(fields, _) => {
                Box::new(fields.iter().enumerate().map(|(i, field)| (i.to_string(), field)))
            }
            other => panic!("Tried to iterate over the fields of '{other:?}', which has none"),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            HirPattern::Identifier(ident) => ident.location.span,
            HirPattern::Mutable(_, location)
            | HirPattern::Tuple(_, location)
            | HirPattern::Struct(_, _, location) => location.span,
        }
    }

    pub fn location(&self) -> Location {
        match self {
            HirPattern::Identifier(ident) => ident.location,
            HirPattern::Mutable(_, location)
            | HirPattern::Tuple(_, location)
            | HirPattern::Struct(_, _, location) => *location,
        }
    }
}

/// Represents an Ast form that can be assigned to. These
/// can be found on the left hand side of an assignment `=`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HirLValue {
    Ident(HirIdent, Type),
    MemberAccess {
        object: Box<HirLValue>,
        field_name: Ident,
        field_index: Option<usize>,
        typ: Type,
        location: Location,
    },
    Index {
        array: Box<HirLValue>,
        index: ExprId,
        typ: Type,
        location: Location,
    },
    Dereference {
        lvalue: Box<HirLValue>,
        element_type: Type,
        location: Location,
    },
}
