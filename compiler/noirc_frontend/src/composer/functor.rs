//! This module defines a Functor version of the Ast where
//! each recursive Ast field is replaced by a generic result value.
//! This is given to name resolution, type checking, and the comptime
//! interpreter to prevent them from recursing on ast nodes themselves
//! and falling out of lock-step with each other.
use acvm::FieldElement;
use noirc_errors::Span;

use crate::{macros_api::{Path, UnaryOp, Ident, UnresolvedType, ItemVisibility, Visibility, Pattern, SecondaryAttribute}, ast::{BinaryOp, UnresolvedGenerics, UnresolvedTraitConstraint, ConstrainKind}, token::Attributes};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression<T> {
    Literal(Literal<T>),
    Block(BlockExpression<T>),
    Prefix(PrefixExpression<T>),
    Index(IndexExpression<T>),
    Call(CallExpression<T>),
    MethodCall(MethodCallExpression<T>),
    Constructor(ConstructorExpression<T>),
    MemberAccess(MemberAccessExpression<T>),
    Cast(CastExpression<T>),
    Infix(InfixExpression<T>),
    If(IfExpression<T>),
    Variable(Path),
    Tuple(Vec<T>),
    Lambda(Lambda<T>),
    Quote(BlockExpression<T>),
    Comptime(BlockExpression<T>),
    Error,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Literal<T> {
    Array(ArrayLiteral<T>),
    Slice(ArrayLiteral<T>),
    Bool(bool),
    Integer(FieldElement, /*sign*/ bool), // false for positive integer and true for negative
    Str(String),
    RawStr(String, u8),
    FmtStr(String),
    Unit,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrefixExpression<T> {
    pub operator: UnaryOp,
    pub rhs: T,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InfixExpression<T> {
    pub lhs: T,
    pub operator: BinaryOp,
    pub rhs: T,
}

// This is an infix expression with 'as' as the binary operator
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CastExpression<T> {
    pub lhs: T,
    pub r#type: UnresolvedType,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IfExpression<T> {
    pub condition: T,
    pub consequence: T,
    pub alternative: Option<T>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Lambda<T> {
    pub parameters: Vec<(Pattern, UnresolvedType)>,
    pub return_type: UnresolvedType,
    pub body: T,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionDefinition<T> {
    pub name: Ident,

    // The `Attributes` container holds both `primary` (ones that change the function kind)
    // and `secondary` attributes (ones that do not change the function kind)
    pub attributes: Attributes,

    /// True if this function was defined with the 'unconstrained' keyword
    pub is_unconstrained: bool,

    /// True if this function was defined with the 'comptime' keyword
    pub is_comptime: bool,

    /// Indicate if this function was defined with the 'pub' keyword
    pub visibility: ItemVisibility,

    pub generics: UnresolvedGenerics,
    pub parameters: Vec<Param>,
    pub body: BlockExpression<T>,
    pub span: Span,
    pub where_clause: Vec<UnresolvedTraitConstraint>,
    pub return_type: FunctionReturnType,
    pub return_visibility: Visibility,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Param {
    pub visibility: Visibility,
    pub pattern: Pattern,
    pub typ: UnresolvedType,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FunctionReturnType {
    /// Returns type is not specified.
    Default(Span),
    /// Everything else.
    Ty(UnresolvedType),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ArrayLiteral<T> {
    Standard(Vec<T>),
    Repeated { repeated_element: T, length: T },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CallExpression<T> {
    pub func: T,
    pub arguments: Vec<T>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MethodCallExpression<T> {
    pub object: T,
    pub method_name: Ident,
    pub arguments: Vec<T>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConstructorExpression<T> {
    pub type_name: Path,
    pub fields: Vec<(Ident, T)>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MemberAccessExpression<T> {
    pub lhs: T,
    pub rhs: Ident,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IndexExpression<T> {
    pub collection: T,
    pub index: T,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockExpression<T> {
    pub statements: Vec<T>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Statement<T> {
    Let(LetStatement<T>),
    Constrain(ConstrainStatement<T>),
    Expression(T),
    Assign(AssignStatement<T>),
    For(ForLoopStatement<T>),
    Break,
    Continue,
    Comptime(T),
    Semi(T),
    Error,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LetStatement<T> {
    pub pattern: Pattern,
    pub r#type: UnresolvedType,
    pub expression: T,
    pub attributes: Vec<SecondaryAttribute>,

    // True if this should only be run during compile-time
    pub comptime: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AssignStatement<T> {
    pub lvalue: LValue<T>,
    pub expression: T,
}

/// Represents an Ast form that can be assigned to
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LValue<T> {
    Ident(Ident),
    MemberAccess { object: Box<LValue<T>>, field_name: Ident, span: Span },
    Index { array: Box<LValue<T>>, index: T, span: Span },
    Dereference(Box<LValue<T>>, Span),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConstrainStatement<T> {
    pub condition: T,
    pub message: Option<T>,
    pub kind: ConstrainKind,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ForRange<T> {
    Range { start: T, end: T },
    Array(T),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ForLoopStatement<T> {
    pub identifier: Ident,
    pub range: ForRange<T>,
    pub block: T,
    pub span: Span,
}
