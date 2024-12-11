use std::fmt::Display;

use acvm::FieldElement;
use iter_extended::vecmap;
use noirc_errors::{
    debug_info::{DebugFunctions, DebugTypes, DebugVariables},
    Location,
};

use crate::{
    ast::{BinaryOpKind, IntegerBitSize, Signedness, Visibility},
    token::{Attributes, FunctionAttribute},
};
use crate::{hir_def::function::FunctionSignature, token::FmtStrFragment};
use serde::{Deserialize, Serialize};

use super::HirType;

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
#[derive(Debug, Clone, Hash)]
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
    Constrain(Box<Expression>, Location, Option<Box<(Expression, HirType)>>),
    Assign(Assign),
    Semi(Box<Expression>),
    Break,
    Continue,
}

impl Expression {
    pub fn is_array_or_slice_literal(&self) -> bool {
        matches!(self, Expression::Literal(Literal::Array(_) | Literal::Slice(_)))
    }
}

/// A definition is either a local (variable), function, or is a built-in
/// function that will be generated or referenced by the compiler later.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Definition {
    Local(LocalId),
    Function(FuncId),
    Builtin(String),
    LowLevel(String),
    // used as a foreign/externally defined unconstrained function
    Oracle(String),
}

/// ID of a local definition, e.g. from a let binding or
/// function parameter that should be compiled before it is referenced.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

/// A function ID corresponds directly to an index of `Program::functions`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FuncId(pub u32);

impl Display for FuncId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Hash)]
pub struct Ident {
    pub location: Option<Location>,
    pub definition: Definition,
    pub mutable: bool,
    pub name: String,
    pub typ: Type,
}

#[derive(Debug, Clone, Hash)]
pub struct For {
    pub index_variable: LocalId,
    pub index_name: String,
    pub index_type: Type,

    pub start_range: Box<Expression>,
    pub end_range: Box<Expression>,
    pub block: Box<Expression>,

    pub start_range_location: Location,
    pub end_range_location: Location,
}

#[derive(Debug, Clone, Hash)]
pub enum Literal {
    Array(ArrayLiteral),
    Slice(ArrayLiteral),
    Integer(FieldElement, /*sign*/ bool, Type, Location), // false for positive integer and true for negative
    Bool(bool),
    Unit,
    Str(String),
    FmtStr(Vec<FmtStrFragment>, u64, Box<Expression>),
}

#[derive(Debug, Clone, Hash)]
pub struct Unary {
    pub operator: crate::ast::UnaryOp,
    pub rhs: Box<Expression>,
    pub result_type: Type,
    pub location: Location,
}

pub type BinaryOp = BinaryOpKind;

#[derive(Debug, Clone, Hash)]
pub struct Binary {
    pub lhs: Box<Expression>,
    pub operator: BinaryOp,
    pub rhs: Box<Expression>,
    pub location: Location,
}

#[derive(Debug, Clone)]
pub struct Lambda {
    pub function: Ident,
    pub env: Ident,
}

#[derive(Debug, Clone, Hash)]
pub struct If {
    pub condition: Box<Expression>,
    pub consequence: Box<Expression>,
    pub alternative: Option<Box<Expression>>,
    pub typ: Type,
}

#[derive(Debug, Clone, Hash)]
pub struct Cast {
    pub lhs: Box<Expression>,
    pub r#type: Type,
    pub location: Location,
}

#[derive(Debug, Clone, Hash)]
pub struct ArrayLiteral {
    pub contents: Vec<Expression>,
    pub typ: Type,
}

#[derive(Debug, Clone, Hash)]
pub struct Call {
    pub func: Box<Expression>,
    pub arguments: Vec<Expression>,
    pub return_type: Type,
    pub location: Location,
}

#[derive(Debug, Clone, Hash)]
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
#[derive(Debug, Clone, Hash)]
pub struct Let {
    pub id: LocalId,
    pub mutable: bool,
    pub name: String,
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone, Hash)]
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
#[derive(Debug, Clone, Hash)]
pub enum LValue {
    Ident(Ident),
    Index { array: Box<LValue>, index: Box<Expression>, element_type: Type, location: Location },
    MemberAccess { object: Box<LValue>, field_index: usize },
    Dereference { reference: Box<LValue>, element_type: Type },
}

pub type Parameters = Vec<(LocalId, /*mutable:*/ bool, /*name:*/ String, Type)>;

/// Represents how an Acir function should be inlined.
/// This type is only relevant for ACIR functions as we do not inline any Brillig functions
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum InlineType {
    /// The most basic entry point can expect all its functions to be inlined.
    /// All function calls are expected to be inlined into a single ACIR.
    #[default]
    Inline,
    /// Functions marked as inline always will always be inlined, even in brillig contexts.
    InlineAlways,
    /// Functions marked as foldable will not be inlined and compiled separately into ACIR
    Fold,
    /// Functions marked to have no predicates will not be inlined in the default inlining pass
    /// and will be separately inlined after the flattening pass.
    /// They are different from `Fold` as they are expected to be inlined into the program
    /// entry point before being used in the backend.
    /// This attribute is unsafe and can cause a function whose logic relies on predicates from
    /// the flattening pass to fail.
    NoPredicates,
}

impl From<&Attributes> for InlineType {
    fn from(attributes: &Attributes) -> Self {
        attributes.function().map_or(InlineType::default(), |func_attribute| match func_attribute {
            FunctionAttribute::Fold => InlineType::Fold,
            FunctionAttribute::NoPredicates => InlineType::NoPredicates,
            FunctionAttribute::InlineAlways => InlineType::InlineAlways,
            _ => InlineType::default(),
        })
    }
}

impl InlineType {
    pub fn is_entry_point(&self) -> bool {
        match self {
            InlineType::Inline => false,
            InlineType::InlineAlways => false,
            InlineType::Fold => true,
            InlineType::NoPredicates => false,
        }
    }
}

impl std::fmt::Display for InlineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InlineType::Inline => write!(f, "inline"),
            InlineType::InlineAlways => write!(f, "inline_always"),
            InlineType::Fold => write!(f, "fold"),
            InlineType::NoPredicates => write!(f, "no_predicates"),
        }
    }
}

#[derive(Debug, Clone, Hash)]
pub struct Function {
    pub id: FuncId,
    pub name: String,

    pub parameters: Parameters,
    pub body: Expression,

    pub return_type: Type,
    pub unconstrained: bool,
    pub inline_type: InlineType,
    pub func_sig: FunctionSignature,
}

/// Compared to hir_def::types::Type, this monomorphized Type has:
/// - All type variables and generics removed
/// - Concrete lengths for each array and string
/// - Several other variants removed (such as Type::Constant)
/// - All structs replaced with tuples
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Type {
    Field,
    Array(/*len:*/ u32, Box<Type>), // Array(4, Field) = [Field; 4]
    Integer(Signedness, /*bits:*/ IntegerBitSize), // u32 = Integer(unsigned, ThirtyTwo)
    Bool,
    String(/*len:*/ u32), // String(4) = str[4]
    FmtString(/*len:*/ u32, Box<Type>),
    Unit,
    Tuple(Vec<Type>),
    Slice(Box<Type>),
    MutableReference(Box<Type>),
    Function(
        /*args:*/ Vec<Type>,
        /*ret:*/ Box<Type>,
        /*env:*/ Box<Type>,
        /*unconstrained:*/ bool,
    ),
}

impl Type {
    pub fn flatten(&self) -> Vec<Type> {
        match self {
            Type::Tuple(fields) => fields.iter().flat_map(|field| field.flatten()).collect(),
            _ => vec![self.clone()],
        }
    }
}

#[derive(Debug, Clone, Hash)]
pub struct Program {
    pub functions: Vec<Function>,
    pub function_signatures: Vec<FunctionSignature>,
    pub main_function_signature: FunctionSignature,
    pub return_location: Option<Location>,
    pub return_visibility: Visibility,
    pub debug_variables: DebugVariables,
    pub debug_functions: DebugFunctions,
    pub debug_types: DebugTypes,
}

impl Program {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        functions: Vec<Function>,
        function_signatures: Vec<FunctionSignature>,
        main_function_signature: FunctionSignature,
        return_location: Option<Location>,
        return_visibility: Visibility,
        debug_variables: DebugVariables,
        debug_functions: DebugFunctions,
        debug_types: DebugTypes,
    ) -> Program {
        Program {
            functions,
            function_signatures,
            main_function_signature,
            return_location,
            return_visibility,
            debug_variables,
            debug_functions,
            debug_types,
        }
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

    /// Takes a function body by replacing it with `false` and
    /// returning the previous value
    pub fn take_function_body(&mut self, function: FuncId) -> Expression {
        let function_definition = &mut self[function];
        let replacement = Expression::Block(vec![]);
        std::mem::replace(&mut function_definition.body, replacement)
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
            Type::String(len) => write!(f, "str<{len}>"),
            Type::FmtString(len, elements) => {
                write!(f, "fmtstr<{len}, {elements}>")
            }
            Type::Unit => write!(f, "()"),
            Type::Tuple(elements) => {
                let elements = vecmap(elements, ToString::to_string);
                write!(f, "({})", elements.join(", "))
            }
            Type::Function(args, ret, env, unconstrained) => {
                if *unconstrained {
                    write!(f, "unconstrained ")?;
                }

                let args = vecmap(args, ToString::to_string);
                let closure_env_text = match **env {
                    Type::Unit => "".to_string(),
                    _ => format!(" with closure environment {env}"),
                };
                write!(f, "fn({}) -> {}{}", args.join(", "), ret, closure_env_text)
            }
            Type::Slice(element) => write!(f, "[{element}]"),
            Type::MutableReference(element) => write!(f, "&mut {element}"),
        }
    }
}
