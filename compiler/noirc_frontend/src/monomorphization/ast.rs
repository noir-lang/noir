use std::{borrow::Cow, collections::BTreeMap, fmt::Display};

use iter_extended::vecmap;
use noirc_artifacts::debug::{DebugFunctions, DebugTypes, DebugVariables};
use noirc_errors::Location;

use crate::{
    ast::{BinaryOpKind, IntegerBitSize},
    hir_def::expr::Constructor,
    shared::Signedness,
    signed_field::SignedField,
    token::Attributes,
};
use crate::{hir_def::function::FunctionSignature, token::FmtStrFragment};
use crate::{shared::Visibility, token::FunctionAttributeKind};
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
    Loop(Box<Expression>),
    While(While),
    If(If),
    Match(Match),
    Tuple(Vec<Expression>),
    ExtractTupleField(Box<Expression>, usize),
    Call(Call),
    Let(Let),
    Constrain(Box<Expression>, Location, Option<Box<(Expression, HirType)>>),
    Assign(Assign),
    Semi(Box<Expression>),
    Clone(Box<Expression>),
    Drop(Box<Expression>),
    Break,
    Continue,
}

impl Expression {
    pub fn is_array_or_vector_literal(&self) -> bool {
        matches!(self, Expression::Literal(Literal::Array(_) | Literal::Vector(_)))
    }

    /// The return type of an expression, if it has an obvious one.
    pub fn return_type(&self) -> Option<Cow<Type>> {
        fn borrowed(typ: &Type) -> Option<Cow<Type>> {
            Some(Cow::Borrowed(typ))
        }
        let owned = |typ: Type| Some(Cow::Owned(typ));

        match self {
            Expression::Ident(ident) => borrowed(&ident.typ),
            Expression::Literal(literal) => match literal {
                Literal::Array(literal) | Literal::Vector(literal) => borrowed(&literal.typ),
                Literal::Integer(_, typ, _) => borrowed(typ),
                Literal::Bool(_) => borrowed(&Type::Bool),
                Literal::Unit => borrowed(&Type::Unit),
                Literal::Str(s) => owned(Type::String(s.len() as u32)),
                Literal::FmtStr(_, size, expr) => expr.return_type().and_then(|typ| {
                    owned(Type::FmtString(*size as u32, Box::new(typ.into_owned())))
                }),
            },
            Expression::Block(xs) => xs.last().and_then(|x| x.return_type()),
            Expression::Unary(unary) => borrowed(&unary.result_type),
            Expression::Binary(binary) => {
                if binary.operator.is_comparator() {
                    borrowed(&Type::Bool)
                } else {
                    binary.lhs.return_type()
                }
            }
            Expression::Index(index) => borrowed(&index.element_type),
            Expression::Cast(cast) => borrowed(&cast.r#type),
            Expression::If(if_) => borrowed(&if_.typ),
            Expression::ExtractTupleField(x, idx) => match x.as_ref() {
                Expression::Tuple(xs) => {
                    assert!(xs.len() > *idx, "index out of bounds in tuple return type");
                    xs[*idx].return_type()
                }
                x => {
                    let typ = x.return_type()?;
                    let Type::Tuple(types) = typ.as_ref() else {
                        unreachable!("unexpected type for tuple field extraction: {typ}");
                    };
                    assert!(types.len() > *idx, "index out of bounds in tuple return type");
                    owned(types[*idx].clone())
                }
            },
            Expression::Clone(x) => x.return_type(),
            Expression::Call(call) => borrowed(&call.return_type),
            Expression::Match(m) => borrowed(&m.typ),

            Expression::Tuple(xs) => {
                let types = xs
                    .iter()
                    .filter_map(|x| x.return_type())
                    .map(|t| t.into_owned())
                    .collect::<Vec<_>>();
                if types.len() != xs.len() {
                    return None;
                }
                owned(Type::Tuple(types))
            }

            Expression::For(_)
            | Expression::Loop(_)
            | Expression::While(_)
            | Expression::Let(_)
            | Expression::Constrain(_, _, _)
            | Expression::Assign(_)
            | Expression::Semi(_)
            | Expression::Drop(_)
            | Expression::Break
            | Expression::Continue => None,
        }
    }

    /// Check if the expression will need to have its type deduced from a literal,
    /// which could be ambiguous.
    ///
    /// For example:
    /// ```ignore
    /// let a = 1;
    /// let b = if (a > 0) { 2 } else { 3 };
    /// ```
    pub fn needs_type_inference_from_literal(&self) -> bool {
        match self {
            Expression::Literal(_) => true,

            Expression::Block(expressions) => expressions
                .last()
                .map(|x| x.needs_type_inference_from_literal())
                .unwrap_or_default(),

            Expression::Unary(unary) => unary.rhs.needs_type_inference_from_literal(),

            Expression::Binary(binary) => {
                if binary.operator.is_comparator() {
                    false
                } else {
                    binary.lhs.needs_type_inference_from_literal()
                }
            }
            Expression::If(if_) => {
                if_.consequence.needs_type_inference_from_literal()
                    && if_
                        .alternative
                        .as_ref()
                        .map(|x| x.needs_type_inference_from_literal())
                        .unwrap_or_default()
            }
            Expression::Match(m) => {
                m.cases.iter().all(|c| c.branch.needs_type_inference_from_literal())
                    && m.default_case.as_ref().is_none_or(|x| x.needs_type_inference_from_literal())
            }

            Expression::Tuple(xs) => xs.iter().any(|x| x.needs_type_inference_from_literal()),

            Expression::ExtractTupleField(x, _) => x.needs_type_inference_from_literal(),

            // The following expressions either carry an obvious type, or return nothing.
            Expression::Ident(_)
            | Expression::Call(_)
            | Expression::Index(_)
            | Expression::Cast(_)
            | Expression::For(_)
            | Expression::Loop(_)
            | Expression::While(_)
            | Expression::Let(_)
            | Expression::Constrain(_, _, _)
            | Expression::Assign(_)
            | Expression::Semi(_)
            | Expression::Clone(_)
            | Expression::Drop(_)
            | Expression::Break
            | Expression::Continue => false,
        }
    }
}

/// A definition is either a local (variable), function, or is a built-in
/// function that will be generated or referenced by the compiler later.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Definition {
    Local(LocalId),
    Global(GlobalId),
    Function(FuncId),
    Builtin(String),
    LowLevel(String),
    // used as a foreign/externally defined unconstrained function
    Oracle(String),
}

/// ID of a local definition, e.g. from a let binding or
/// function parameter that should be compiled before it is referenced.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalId(pub u32);

/// A function ID corresponds directly to an index of `Program::globals`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GlobalId(pub u32);

/// A function ID corresponds directly to an index of `Program::functions`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FuncId(pub u32);

/// Each identifier is given a unique ID to distinguish different uses of identifiers.
/// This is used, for example, in last use analysis to determine which identifiers represent
/// the last use of their definition and can thus be moved instead of cloned.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IdentId(pub u32);

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
    pub id: IdentId,
}

#[derive(Debug, Clone, Hash)]
pub struct For {
    pub index_variable: LocalId,
    pub index_name: String,
    pub index_type: Type,

    pub start_range: Box<Expression>,
    pub end_range: Box<Expression>,
    pub block: Box<Expression>,
    pub inclusive: bool,

    pub start_range_location: Location,
    pub end_range_location: Location,
}

#[derive(Debug, Clone, Hash)]
pub struct While {
    pub condition: Box<Expression>,
    pub body: Box<Expression>,
}

#[derive(Debug, Clone, Hash)]
pub enum Literal {
    Array(ArrayLiteral),
    Vector(ArrayLiteral),
    Integer(SignedField, Type, Location),
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

    /// Carried over from `HirPrefixExpression::skip`. This also flags whether we should directly
    /// return `rhs` and skip performing this operation. Compared to replacing this node with rhs
    /// directly, keeping this flag keeps `--show-monomorphized` output the same.
    pub skip: bool,
}

pub type BinaryOp = BinaryOpKind;

#[derive(Debug, Clone, Hash)]
pub struct Binary {
    pub lhs: Box<Expression>,
    pub operator: BinaryOp,
    pub rhs: Box<Expression>,
    pub location: Location,
}

#[derive(Debug, Clone, Hash)]
pub struct If {
    pub condition: Box<Expression>,
    pub consequence: Box<Expression>,
    pub alternative: Option<Box<Expression>>,
    pub typ: Type,
}

#[derive(Debug, Clone, Hash)]
pub struct Match {
    pub variable_to_match: (LocalId, String),
    pub cases: Vec<MatchCase>,
    pub default_case: Option<Box<Expression>>,
    pub typ: Type,
}

#[derive(Debug, Clone, Hash)]
pub struct MatchCase {
    pub constructor: Constructor,
    pub arguments: Vec<(LocalId, String)>,
    pub branch: Expression,
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
    Index {
        array: Box<LValue>,
        index: Box<Expression>,
        element_type: Type,
        location: Location,
    },
    MemberAccess {
        object: Box<LValue>,
        field_index: usize,
    },
    Dereference {
        reference: Box<LValue>,
        element_type: Type,
    },
    /// Analogous to Expression::Clone. Clone the resulting lvalue after evaluating it.
    Clone(Box<LValue>),
}

pub type Parameters =
    Vec<(LocalId, /*mutable:*/ bool, /*name:*/ String, Type, Visibility)>;

/// Represents how an Acir function should be inlined.
/// This type is only relevant for ACIR functions as we do not inline any Brillig functions
#[derive(
    Default, Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
pub enum InlineType {
    /// The most basic entry point can expect all its functions to be inlined.
    /// All function calls are expected to be inlined into a single ACIR.
    #[default]
    Inline,
    /// Functions marked as inline always will always be inlined, even in brillig contexts.
    InlineAlways,
    /// Functions marked as inline never will never be inlined
    InlineNever,
    /// Functions marked as foldable will not be inlined and compiled separately into ACIR
    Fold,
    /// Functions marked to have no predicates will not be inlined in the default inlining pass
    /// and will be separately inlined after the flattening pass.
    ///
    /// Flattening and inlining are necessary compiler passes in the ACIR runtime. More specifically,
    /// flattening is the removal of control flow through predicating side-effectual instructions.
    /// In some cases, a user may only want predicates applied to the result of a function call rather
    /// than all of a function's internal execution. To allow this behavior, we can simply inline a function
    /// after performing flattening (as ultimately in ACIR a non-entry point function will have to be inlined).
    /// These functions are different from `Fold` as they are expected to be inlined into the program
    /// entry point before being used in the backend.
    ///
    /// This attribute is unsafe and can cause a function whose logic relies on predicates from
    /// the flattening pass to fail.
    NoPredicates,
}

impl From<&Attributes> for InlineType {
    fn from(attributes: &Attributes) -> Self {
        attributes.function().map_or(InlineType::default(), |func_attribute| match &func_attribute
            .kind
        {
            FunctionAttributeKind::Fold => InlineType::Fold,
            FunctionAttributeKind::NoPredicates => InlineType::NoPredicates,
            FunctionAttributeKind::InlineAlways => InlineType::InlineAlways,
            FunctionAttributeKind::InlineNever => InlineType::InlineNever,
            _ => InlineType::default(),
        })
    }
}

impl InlineType {
    pub fn is_entry_point(&self) -> bool {
        match self {
            InlineType::Inline => false,
            InlineType::InlineAlways => false,
            InlineType::InlineNever => false,
            InlineType::Fold => true,
            InlineType::NoPredicates => false,
        }
    }

    /// Produce an `InlineType` which we can use with an unconstrained version of a function.
    pub fn into_unconstrained(self) -> Self {
        match self {
            InlineType::Inline | InlineType::InlineAlways | InlineType::InlineNever => self,
            InlineType::Fold => {
                // The #[fold] attribute is about creating separate ACIR circuits for proving,
                // not relevant in Brillig. Leaving it violates some expectations that each
                // will become its own entry point.
                Self::default()
            }
            InlineType::NoPredicates => {
                // The #[no_predicates] are guaranteed to be inlined after flattening,
                // which is needed for some of the programs even in Brillig, otherwise
                // some intrinsics can survive until Brillig-gen that weren't supposed to.
                // We can keep these, or try inlining more aggressively, since we don't
                // have to wait until after flattening in Brillig, but InlineAlways
                // resulted in some Brillig bytecode size regressions.
                self
            }
        }
    }
}

impl Display for InlineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InlineType::Inline => write!(f, "inline"),
            InlineType::InlineAlways => write!(f, "inline_always"),
            InlineType::InlineNever => write!(f, "inline_never"),
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
    pub return_visibility: Visibility,
    pub unconstrained: bool,
    pub inline_type: InlineType,
    pub func_sig: FunctionSignature,
}

/// Compared to hir_def::types::Type, this monomorphized Type has:
/// - All type variables and generics removed
/// - Concrete lengths for each array and string
/// - Several other variants removed (such as Type::Constant)
/// - All structs replaced with tuples
#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub enum Type {
    Field,
    Array(/*len:*/ u32, Box<Type>), // Array(4, Field) = [Field; 4]
    Integer(Signedness, /*bits:*/ IntegerBitSize), // u32 = Integer(unsigned, ThirtyTwo)
    Bool,
    String(/*len:*/ u32), // String(4) = str[4]
    FmtString(/*len:*/ u32, Box<Type>),
    Unit,
    Tuple(Vec<Type>),
    Vector(Box<Type>),
    Reference(Box<Type>, /*mutable:*/ bool),
    /// `(args, ret, env, unconstrained)`
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

    /// Returns the element type of this array or vector
    pub fn array_element_type(&self) -> Option<&Type> {
        match self {
            Type::Array(_, elem) | Type::Vector(elem) => Some(elem),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Hash, Default)]
pub struct Program {
    pub functions: Vec<Function>,
    pub function_signatures: Vec<FunctionSignature>,
    pub main_function_signature: FunctionSignature,
    pub return_location: Option<Location>,
    pub globals: BTreeMap<GlobalId, (String, Type, Expression)>,
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
        globals: BTreeMap<GlobalId, (String, Type, Expression)>,
        debug_variables: DebugVariables,
        debug_functions: DebugFunctions,
        debug_types: DebugTypes,
    ) -> Program {
        Program {
            functions,
            function_signatures,
            main_function_signature,
            return_location,
            globals,
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

    /// Globals are expected to be generated within a different context than
    /// all other functions in the program. Thus, the globals space has the same
    /// ID as `main`, although we should never expect a clash in these IDs.
    pub fn global_space_id() -> FuncId {
        FuncId(0)
    }

    /// Takes a function body by replacing it with `false` and
    /// returning the previous value
    pub fn take_function_body(&mut self, function: FuncId) -> Expression {
        let function_definition = &mut self[function];
        let replacement = Expression::Block(vec![]);
        std::mem::replace(&mut function_definition.body, replacement)
    }

    pub fn return_visibility(&self) -> Visibility {
        self.main().return_visibility
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

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        super::printer::AstPrinter::default().print_program(self, f)
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        super::printer::AstPrinter::default().print_function(
            self,
            f,
            super::printer::FunctionPrintOptions::default(),
        )
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        super::printer::AstPrinter::default().print_expr(self, f)
    }
}

impl Display for Type {
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
                if elements.len() == 1 {
                    write!(f, "({},)", elements[0])
                } else {
                    write!(f, "({})", elements.join(", "))
                }
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
            Type::Vector(element) => write!(f, "[{element}]"),
            Type::Reference(element, mutable) if *mutable => write!(f, "&mut {element}"),
            Type::Reference(element, _mutable) => write!(f, "&{element}"),
        }
    }
}
