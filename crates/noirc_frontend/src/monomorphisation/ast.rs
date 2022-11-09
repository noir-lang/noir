use acvm::FieldElement;
use noirc_abi::Abi;
use noirc_errors::Location;

use crate::{util::vecmap, BinaryOpKind, Signedness};

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

/// TODO: Ssa doesn't support these yet.
#[derive(Debug, Clone)]
pub struct CallBuiltin {
    pub opcode: String,
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
    MemberAccess { object: Box<LValue>, field_index: usize },
}

#[derive(Debug, Clone)]
pub struct Function {
    pub id: FuncId,
    pub name: String,

    pub parameters: Vec<(DefinitionId, /*mutable:*/ bool, /*name:*/ String, Type)>,
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

impl Type {
    pub fn flatten(&self) -> Vec<Type> {
        match self {
            Type::Tuple(fields) => fields.iter().flat_map(|field| field.flatten()).collect(),
            _ => vec![self.clone()],
        }
    }
}

pub struct Program {
    pub functions: Vec<Function>,
    pub abi: Abi,
}

impl Program {
    pub fn new(main: Function, abi: Abi) -> Program {
        Program { functions: vec![main], abi }
    }

    pub fn push_function(&mut self, function: Function) {
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

impl std::ops::Index<FuncId> for Program {
    type Output = Function;

    fn index(&self, index: FuncId) -> &Self::Output {
        &self.functions[index.0 as usize]
    }
}

impl std::ops::IndexMut<FuncId> for Program {
    fn index_mut(&mut self, index: FuncId) -> &mut Self::Output {
        &mut self.functions[index.0 as usize]
    }
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for function in &self.functions {
            super::printer::AstPrinter::default().print_function(function, f)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        super::printer::AstPrinter::default().print_function(self, f)
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        super::printer::AstPrinter::default().print_expr(self, f)
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Field => write!(f, "Field"),
            Type::Array(len, elems) => write!(f, "[{}; {}]", elems, len),
            Type::Integer(sign, bits) => match sign {
                Signedness::Unsigned => write!(f, "u{}", bits),
                Signedness::Signed => write!(f, "i{}", bits),
            },
            Type::Bool => write!(f, "bool"),
            Type::Unit => write!(f, "()"),
            Type::Tuple(elems) => {
                let elems = vecmap(elems, ToString::to_string);
                write!(f, "({})", elems.join(", "))
            }
        }
    }
}
