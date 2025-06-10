use super::expr::HirIdent;
use crate::Type;
use crate::ast::{Ident, TupleWithDoubleDot};
use crate::node_interner::{ExprId, StmtId};
use crate::token::SecondaryAttribute;
use noirc_errors::{Location, Span};

/// A HirStatement is the result of performing name resolution on
/// the Statement AST node. Unlike the AST node, any nested nodes
/// are referred to indirectly via ExprId or StmtId, which can be
/// used to retrieve the relevant node via the NodeInterner.
#[derive(Debug, Clone)]
pub enum HirStatement {
    Let(HirLetStatement),
    Assign(HirAssignStatement),
    For(HirForStatement),
    Loop(ExprId),
    While(ExprId, ExprId),
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

    /// Creates a new 'basic' let statement with no attributes and is not comptime nor global.
    pub fn basic(pattern: HirPattern, r#type: Type, expression: ExprId) -> HirLetStatement {
        Self::new(pattern, r#type, expression, Vec::new(), false, false)
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum HirPattern {
    Identifier(HirIdent),
    Mutable(Box<HirPattern>, Location),
    Tuple(Vec<HirPattern>, Location),
    TupleWithDoubleDot(TupleWithDoubleDot<HirPattern>),
    Struct(Type, Vec<(Ident, HirPattern)>, Location),
}

impl HirPattern {
    pub fn span(&self) -> Span {
        self.location().span
    }

    pub fn location(&self) -> Location {
        match self {
            HirPattern::Identifier(ident) => ident.location,
            HirPattern::Mutable(_, location)
            | HirPattern::Tuple(_, location)
            | HirPattern::Struct(_, _, location) => *location,
            HirPattern::TupleWithDoubleDot(tuple) => tuple.location,
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
        /// `index` is required to be an identifier to simplify sequencing of side-effects.
        /// However we also store types and locations on ExprIds which makes these necessary
        /// for evaluating/compiling HirIdents so we don't directly require a HirIdent type here.
        index: ExprId,
        typ: Type,
        location: Location,
    },
    Dereference {
        lvalue: Box<HirLValue>,
        element_type: Type,
        implicitly_added: bool,
        location: Location,
    },
}
