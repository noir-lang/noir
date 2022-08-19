use acvm::FieldElement;
use fm::FileId;
use noirc_errors::Location;

use crate::{Signedness, hir_def::expr::HirBinaryOp};

#[derive(Debug, Clone)]
pub enum Expression {
    Ident(Ident),
    Literal(Literal),
    Block(Vec<Expression>),
    Unary(Unary),
    Binary(Binary),
    Index(Index),
    Call(Call),
    Cast(Cast),
    For(For),
    If(If),
    Tuple(Vec<Expression>),
    ExtractTupleField(Box<Expression>, usize),

    Let(Let),
    Constrain(Box<Expression>, FileId),
    Assign(Assign),
    Semi(Box<Expression>),
}

#[derive(Debug, Copy, Clone)]
pub struct DefinitionId(pub u32);

#[derive(Debug, Copy, Clone)]
pub struct FuncId(pub u32);

#[derive(Debug, Clone)]
pub struct Ident {
    pub location: Option<Location>,
    pub id: DefinitionId,
}

#[derive(Debug, Clone)]
pub struct For {
    pub index_variable: DefinitionId,
    pub start_range: Box<Expression>,
    pub end_range: Box<Expression>,
    pub block: Box<Expression>,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Array(ArrayLiteral),
    Integer(FieldElement),
    Bool(bool),
    Str(String),
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub operator: crate::UnaryOp,
    pub rhs: Box<Expression>,
}

pub type BinaryOp = HirBinaryOp;

#[derive(Debug, Clone)]
pub struct Binary {
    pub lhs: Box<Expression>,
    pub operator: BinaryOp,
    pub rhs: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct If {
    pub condition: Box<Expression>,
    pub consequence: Box<Expression>,
    pub alternative: Option<Box<Expression>>,
}

#[derive(Debug, Clone)]
pub struct Cast {
    pub lhs: Box<Expression>,
    pub r#type: Type,
}

#[derive(Debug, Clone)]
pub struct ArrayLiteral {
    pub length: u128,
    pub contents: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub func_id: FuncId,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct Index {
    pub collection: Box<Expression>,
    pub index: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct Let {
    pub id: DefinitionId,
    pub r#type: Type,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub lvalue: LValue,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct BinaryStatement {
    pub lhs: Box<Expression>,
    pub r#type: Type,
    pub expression: Box<Expression>,
}

/// Represents an Ast form that can be assigned to
#[derive(Debug, Clone)]
pub enum LValue {
    Ident(Ident),
    Index { array: Box<LValue>, index: Box<Expression> },
}

#[derive(Debug, Clone)]
pub struct Function {
    pub id: FuncId,
    pub parameters: Vec<(DefinitionId, Type)>,
    pub body: Expression,
}

/// A monomorphised Type has all type variables removed
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Field,
    Array(/*len:*/ u64, Box<Type>),     // Array(4, Field) = [Field; 4]
    Integer(Signedness, /*bits:*/ u32), // u32 = Integer(unsigned, 32)
    Bool,
    Unit,
    Tuple(Vec<Type>),
}
