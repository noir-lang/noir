use std::rc::Rc;

use acvm::FieldElement;
use fm::FileId;
use noirc_errors::Location;

use crate::{BinaryOpKind, Signedness};

#[derive(Debug, Clone)]
pub enum Expression {
    Ident(Ident),
    Literal(Literal),
    Block(Block),
    Unary(Unary),
    Binary(Binary),
    Index(Index),
    Call(Call),
    Cast(Cast),
    For(For),
    If(If),

    Let(Let),
    Constrain(Constrain),
    Assign(Assign),
    Semi(Box<Expression>),
}

#[derive(Debug, Copy, Clone)]
pub struct DefinitionId(pub u32);

#[derive(Debug, Copy, Clone)]
pub struct FuncId(pub u32);

#[derive(Debug, Clone)]
pub struct Ident {
    pub location: Location,
    pub id: DefinitionId,
    pub definition: Rc<Expression>,
}

#[derive(Debug, Clone)]
pub struct For {
    pub identifier: Ident,
    pub start_range: Box<Expression>,
    pub end_range: Box<Expression>,
    pub block: Box<Expression>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BinaryOp {
    pub kind: BinaryOpKind,
    pub location: Location,
}

impl BinaryOp {
    pub fn new(op: crate::BinaryOp, file: FileId) -> Self {
        let kind = op.contents;
        let location = Location::new(op.span(), file);
        BinaryOp { location, kind }
    }
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
pub struct Block(pub Vec<Expression>);

#[derive(Debug, Clone)]
pub struct Let {
    pub ident: Ident,
    pub r#type: Type,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub lvalue: LValue,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct Constrain(pub Box<Expression>, pub FileId);

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

/// A monomorphised Type has all type variables,
/// constness, structs, and tuples removed.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Field,
    Array(/*len:*/ u64, Box<Type>), // Array(4, Field) = [Field; 4]
    Integer(Signedness, /*bits:*/ u32), // u32 = Integer(unsigned, 32)
    Bool,
    Unit,
}
