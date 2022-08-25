use acvm::FieldElement;
use noirc_abi::Abi;
use noirc_errors::Location;

use crate::{BinaryOpKind, Signedness};

#[derive(Debug, Clone)]
pub enum Expression {
    Ident(Ident),
    Literal(Literal),
    Block(Vec<Expression>),
    Unary(Unary),
    Binary(Binary),
    Index(Index),
    Cast(Cast),
    For(For),
    If(If),
    Tuple(Vec<Expression>),
    ExtractTupleField(Box<Expression>, usize),
    Call(Call),
    CallBuiltin(CallBuiltin),
    CallLowLevel(CallLowLevel),

    Let(Let),
    Constrain(Box<Expression>, Location),
    Assign(Assign),
    Semi(Box<Expression>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DefinitionId(pub u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FuncId(pub u32);

#[derive(Debug, Clone)]
pub struct Ident {
    pub location: Option<Location>,
    pub id: DefinitionId,
    pub name: String,
    pub typ: Type,
}

#[derive(Debug, Clone)]
pub struct For {
    pub index_variable: DefinitionId,
    pub index_name: String,
    pub index_type: Type,

    pub start_range: Box<Expression>,
    pub end_range: Box<Expression>,
    pub block: Box<Expression>,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Array(ArrayLiteral),
    Integer(FieldElement, Type),
    Bool(bool),
    Str(String),
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub operator: crate::UnaryOp,
    pub rhs: Box<Expression>,
}

pub type BinaryOp = BinaryOpKind;

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
    pub element_type: Type,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub func_id: FuncId,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct CallLowLevel {
    pub opcode: String,
    pub arguments: Vec<Expression>,
}

/// TODO: Ssa doesn't support these yet
#[derive(Debug, Clone)]
pub struct CallBuiltin {
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
    pub name: String,
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
    pub name: String,

    pub parameters: Vec<(DefinitionId, Type, /*name:*/ String)>,
    pub body: Expression,

    pub return_type: Type,
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

pub struct Functions {
    functions: Vec<Function>,
    pub abi: Abi,
}

impl Functions {
    pub fn new(main: Function, abi: Abi) -> Functions {
        Functions { functions: vec![main], abi }
    }

    pub fn push(&mut self, function: Function) {
        self.functions.push(function);
    }

    pub fn main(&mut self) -> &mut Function {
        &mut self.functions[0]
    }

    pub fn take_main_body(&mut self) -> Expression {
        self.take_function_body(FuncId(0))
    }

    /// Takes a function body by replacing it with `false` and
    /// returning the previous value
    pub fn take_function_body(&mut self, function: FuncId) -> Expression {
        let main = &mut self.functions[function.0 as usize];
        let replacement = Expression::Literal(Literal::Bool(false));
        std::mem::replace(&mut main.body, replacement)
    }
}

impl std::ops::Index<FuncId> for Functions {
    type Output = Function;

    fn index(&self, index: FuncId) -> &Self::Output {
        &self.functions[index.0 as usize]
    }
}

impl std::ops::IndexMut<FuncId> for Functions {
    fn index_mut(&mut self, index: FuncId) -> &mut Self::Output {
        &mut self.functions[index.0 as usize]
    }
}
