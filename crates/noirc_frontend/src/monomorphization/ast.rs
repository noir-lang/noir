use acvm::FieldElement;
use iter_extended::vecmap;
use noirc_abi::FunctionSignature;
use noirc_errors::Location;

use crate::{BinaryOpKind, Signedness};

/// The monomorphized AST is expression-based, all statements are also
/// folded into this expression enum. Compared to the HIR, the monomorphized
/// AST has several differences:
/// - It is self-contained and does not require referencing an external interner
/// - All Types used within are monomorphized and no longer contain any generic types
/// - All Patterns are expanded into multiple variables. This means each definition now
///   defines only 1 variable `let a = 1;`, and any that previously defined multiple,
///   e.g. `let (a, b) = (1, 2)` have been split up: `let tmp = (1, 2); let a = tmp.0; let b = tmp.1;`.
///   This also affects function parameters: `fn foo((a, b): (i32, i32)` => `fn foo(a: i32, b: i32)`.
/// - All structs are replaced with tuples
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

    Let(Let),
    Constrain(Box<Expression>, Location),
    Assign(Assign),
    Semi(Box<Expression>),
}

/// A definition is either a local (variable), function, or is a built-in
/// function that will be generated or referenced by the compiler later.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Definition {
    Local(LocalId),
    Function(FuncId),
    Builtin(String),
    LowLevel(String),
}

/// ID of a local definition, e.g. from a let binding or
/// function parameter that should be compiled before it is referenced.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

/// A function ID corresponds directly to an index of `Program::functions`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FuncId(pub u32);

#[derive(Debug, Clone)]
pub struct Ident {
    pub location: Option<Location>,
    pub definition: Definition,
    pub mutable: bool,
    pub name: String,
    pub typ: Type,
}

#[derive(Debug, Clone)]
pub struct For {
    pub index_variable: LocalId,
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
    pub location: Location,
}

#[derive(Debug, Clone)]
pub struct If {
    pub condition: Box<Expression>,
    pub consequence: Box<Expression>,
    pub alternative: Option<Box<Expression>>,
    pub typ: Type,
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
    pub func: Box<Expression>,
    pub arguments: Vec<Expression>,
    pub return_type: Type,
    pub location: Location,
}

#[derive(Debug, Clone)]
pub struct Index {
    pub collection: Box<Expression>,
    pub index: Box<Expression>,
    pub element_type: Type,
    pub location: Location,
}

/// Rather than a Pattern containing possibly several variables, Let now
/// defines a single variable with the given LocalId. By the time this
/// is produced in monomorphization, let-statements with tuple and struct patterns:
/// ```nr
/// let MyStruct { field1, field2 } = get_struct();
/// ```
/// have been desugared into multiple let statements for simplicity:
/// ```nr
/// let tmp = get_struct();
/// let field1 = tmp.0; // the struct has been translated to a tuple as well
/// let field2 = tmp.1;
/// ```
#[derive(Debug, Clone)]
pub struct Let {
    pub id: LocalId,
    pub mutable: bool,
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
    Index { array: Box<LValue>, index: Box<Expression>, element_type: Type, location: Location },
    MemberAccess { object: Box<LValue>, field_index: usize },
}

pub type Parameters = Vec<(LocalId, /*mutable:*/ bool, /*name:*/ String, Type)>;

#[derive(Debug, Clone)]
pub struct Function {
    pub id: FuncId,
    pub name: String,

    pub parameters: Parameters,
    pub body: Expression,

    pub return_type: Type,
    pub unconstrained: bool,
}

/// Compared to hir_def::types::Type, this monomorphized Type has:
/// - All type variables and generics removed
/// - Concrete lengths for each array and string
/// - Several other variants removed (such as Type::Constant)
/// - No CompTime
/// - All structs replaced with tuples
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Field,
    Array(/*len:*/ u64, Box<Type>),     // Array(4, Field) = [Field; 4]
    Integer(Signedness, /*bits:*/ u32), // u32 = Integer(unsigned, 32)
    Bool,
    String(/*len:*/ u64), // String(4) = str[4]
    Unit,
    Tuple(Vec<Type>),
    Vec(Box<Type>),
    Function(/*args:*/ Vec<Type>, /*ret:*/ Box<Type>),
}

impl Type {
    pub fn flatten(&self) -> Vec<Type> {
        match self {
            Type::Tuple(fields) => fields.iter().flat_map(|field| field.flatten()).collect(),
            _ => vec![self.clone()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub main_function_signature: FunctionSignature,
    /// Indicates whether witness indices are allowed to reoccur in the ABI of the resulting ACIR.
    ///
    /// Note: this has no impact on monomorphization, and is simply attached here for ease of
    /// forwarding to the next phase.
    pub return_distinctness: noirc_abi::AbiDistinctness,
}

impl Program {
    pub fn new(
        functions: Vec<Function>,
        main_function_signature: FunctionSignature,
        return_distinctness: noirc_abi::AbiDistinctness,
    ) -> Program {
        Program { functions, main_function_signature, return_distinctness }
    }

    pub fn main(&self) -> &Function {
        &self[Self::main_id()]
    }

    pub fn main_mut(&mut self) -> &mut Function {
        &mut self[Self::main_id()]
    }

    pub fn main_id() -> FuncId {
        FuncId(0)
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
            Type::Array(len, elements) => write!(f, "[{elements}; {len}]"),
            Type::Integer(sign, bits) => match sign {
                Signedness::Unsigned => write!(f, "u{bits}"),
                Signedness::Signed => write!(f, "i{bits}"),
            },
            Type::Bool => write!(f, "bool"),
            Type::String(len) => write!(f, "str[{len}]"),
            Type::Unit => write!(f, "()"),
            Type::Tuple(elements) => {
                let elements = vecmap(elements, ToString::to_string);
                write!(f, "({})", elements.join(", "))
            }
            Type::Function(args, ret) => {
                let args = vecmap(args, ToString::to_string);
                write!(f, "fn({}) -> {}", args.join(", "), ret)
            }
            Type::Vec(element) => write!(f, "Vec<{element}>"),
        }
    }
}
