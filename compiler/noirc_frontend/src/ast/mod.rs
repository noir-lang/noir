//! The submodules of this module define the various data types required to
//! represent Noir's Ast. Of particular importance are ExpressionKind and Statement
//! which can be found in expression.rs and statement.rs respectively.
//!
//! Noir's Ast is produced by the parser and taken as input to name resolution,
//! where it is converted into the Hir (defined in the hir_def module).
mod expression;
mod function;
mod statement;
mod structure;
mod traits;
mod type_alias;

pub use expression::*;
pub use function::*;

use noirc_errors::Span;
use serde::{Deserialize, Serialize};
pub use statement::*;
pub use structure::*;
pub use traits::*;
pub use type_alias::*;

use crate::{
    node_interner::QuotedTypeId,
    parser::{ParserError, ParserErrorReason},
    token::IntType,
    BinaryTypeOperator,
};
use acvm::acir::AcirField;
use iter_extended::vecmap;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Ord, PartialOrd)]
pub enum IntegerBitSize {
    One,
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
}

impl IntegerBitSize {
    pub fn bit_size(&self) -> u8 {
        match self {
            IntegerBitSize::One => 1,
            IntegerBitSize::Eight => 8,
            IntegerBitSize::Sixteen => 16,
            IntegerBitSize::ThirtyTwo => 32,
            IntegerBitSize::SixtyFour => 64,
        }
    }
}

impl IntegerBitSize {
    pub fn allowed_sizes() -> Vec<Self> {
        vec![Self::One, Self::Eight, Self::ThirtyTwo, Self::SixtyFour]
    }
}

impl From<IntegerBitSize> for u32 {
    fn from(size: IntegerBitSize) -> u32 {
        use IntegerBitSize::*;
        match size {
            One => 1,
            Eight => 8,
            Sixteen => 16,
            ThirtyTwo => 32,
            SixtyFour => 64,
        }
    }
}

pub struct InvalidIntegerBitSizeError(pub u32);

impl TryFrom<u32> for IntegerBitSize {
    type Error = InvalidIntegerBitSizeError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        use IntegerBitSize::*;
        match value {
            1 => Ok(One),
            8 => Ok(Eight),
            16 => Ok(Sixteen),
            32 => Ok(ThirtyTwo),
            64 => Ok(SixtyFour),
            _ => Err(InvalidIntegerBitSizeError(value)),
        }
    }
}

impl core::fmt::Display for IntegerBitSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", u32::from(*self))
    }
}

/// The parser parses types as 'UnresolvedType's which
/// require name resolution to resolve any type names used
/// for structs within, but are otherwise identical to Types.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum UnresolvedTypeData {
    FieldElement,
    Array(UnresolvedTypeExpression, Box<UnresolvedType>), // [Field; 4] = Array(4, Field)
    Slice(Box<UnresolvedType>),
    Integer(Signedness, IntegerBitSize), // u32 = Integer(unsigned, ThirtyTwo)
    Bool,
    Expression(UnresolvedTypeExpression),
    String(UnresolvedTypeExpression),
    FormatString(UnresolvedTypeExpression, Box<UnresolvedType>),
    Unit,

    Parenthesized(Box<UnresolvedType>),

    /// A Named UnresolvedType can be a struct type or a type variable
    Named(Path, Vec<UnresolvedType>, /*is_synthesized*/ bool),

    /// A Trait as return type or parameter of function, including its generics
    TraitAsType(Path, Vec<UnresolvedType>),

    /// &mut T
    MutableReference(Box<UnresolvedType>),

    // Note: Tuples have no visibility, instead each of their elements may have one.
    Tuple(Vec<UnresolvedType>),

    Function(
        /*args:*/ Vec<UnresolvedType>,
        /*ret:*/ Box<UnresolvedType>,
        /*env:*/ Box<UnresolvedType>,
        /*unconstrained:*/ bool,
    ),

    /// The type of quoted code for metaprogramming
    Quoted(crate::QuotedType),

    /// An "as Trait" path leading to an associated type.
    /// E.g. `<Foo as Trait>::Bar`
    AsTraitPath(Box<crate::ast::AsTraitPath>),

    /// An already resolved type. These can only be parsed if they were present in the token stream
    /// as a result of being spliced into a macro's token stream input.
    Resolved(QuotedTypeId),

    Unspecified, // This is for when the user declares a variable without specifying it's type
    Error,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct UnresolvedType {
    pub typ: UnresolvedTypeData,

    // The span is None in the cases where the User omitted a type:
    //  fn Foo() {}  --- return type is UnresolvedType::Unit without a span
    //  let x = 100; --- type is UnresolvedType::Unspecified without a span
    pub span: Option<Span>,
}

/// Type wrapper for a member access
pub struct UnaryRhsMemberAccess {
    pub method_or_field: Ident,
    pub method_call: Option<UnaryRhsMethodCall>,
}

pub struct UnaryRhsMethodCall {
    pub turbofish: Option<Vec<UnresolvedType>>,
    pub macro_call: bool,
    pub args: Vec<Expression>,
}

/// The precursor to TypeExpression, this is the type that the parser allows
/// to be used in the length position of an array type. Only constant integers, variables,
/// and numeric binary operators are allowed here.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum UnresolvedTypeExpression {
    Variable(Path),
    Constant(u32, Span),
    BinaryOperation(
        Box<UnresolvedTypeExpression>,
        BinaryTypeOperator,
        Box<UnresolvedTypeExpression>,
        Span,
    ),
}

impl Recoverable for UnresolvedType {
    fn error(span: Span) -> Self {
        UnresolvedType { typ: UnresolvedTypeData::Error, span: Some(span) }
    }
}

impl std::fmt::Display for UnresolvedTypeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use UnresolvedTypeData::*;
        match self {
            FieldElement => write!(f, "Field"),
            Array(len, typ) => write!(f, "[{typ}; {len}]"),
            Slice(typ) => write!(f, "[{typ}]"),
            Integer(sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "i{num_bits}"),
                Signedness::Unsigned => write!(f, "u{num_bits}"),
            },
            Named(s, args, _) => {
                let args = vecmap(args, |arg| ToString::to_string(&arg.typ));
                if args.is_empty() {
                    write!(f, "{s}")
                } else {
                    write!(f, "{}<{}>", s, args.join(", "))
                }
            }
            TraitAsType(s, args) => {
                let args = vecmap(args, |arg| ToString::to_string(&arg.typ));
                if args.is_empty() {
                    write!(f, "impl {s}")
                } else {
                    write!(f, "impl {}<{}>", s, args.join(", "))
                }
            }
            Tuple(elements) => {
                let elements = vecmap(elements, ToString::to_string);
                write!(f, "({})", elements.join(", "))
            }
            Expression(expression) => expression.fmt(f),
            Bool => write!(f, "bool"),
            String(len) => write!(f, "str<{len}>"),
            FormatString(len, elements) => write!(f, "fmt<{len}, {elements}"),
            Function(args, ret, env, unconstrained) => {
                if *unconstrained {
                    write!(f, "unconstrained ")?;
                }

                let args = vecmap(args, ToString::to_string).join(", ");

                match &env.as_ref().typ {
                    UnresolvedTypeData::Unit => {
                        write!(f, "fn({args}) -> {ret}")
                    }
                    UnresolvedTypeData::Tuple(env_types) => {
                        let env_types = vecmap(env_types, |arg| arg.typ.to_string()).join(", ");
                        write!(f, "fn[{env_types}]({args}) -> {ret}")
                    }
                    other => write!(f, "fn[{other}]({args}) -> {ret}"),
                }
            }
            MutableReference(element) => write!(f, "&mut {element}"),
            Quoted(quoted) => write!(f, "{}", quoted),
            Unit => write!(f, "()"),
            Error => write!(f, "error"),
            Unspecified => write!(f, "unspecified"),
            Parenthesized(typ) => write!(f, "({typ})"),
            Resolved(_) => write!(f, "(resolved type)"),
            AsTraitPath(path) => write!(f, "{path}"),
        }
    }
}

impl std::fmt::Display for UnresolvedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.typ.fmt(f)
    }
}

impl std::fmt::Display for UnresolvedTypeExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnresolvedTypeExpression::Variable(name) => name.fmt(f),
            UnresolvedTypeExpression::Constant(x, _) => x.fmt(f),
            UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs, _) => {
                write!(f, "({lhs} {op} {rhs})")
            }
        }
    }
}

impl UnresolvedType {
    pub fn is_synthesized(&self) -> bool {
        match &self.typ {
            UnresolvedTypeData::MutableReference(ty) => ty.is_synthesized(),
            UnresolvedTypeData::Named(_, _, synthesized) => *synthesized,
            _ => false,
        }
    }

    pub fn without_span(typ: UnresolvedTypeData) -> UnresolvedType {
        UnresolvedType { typ, span: None }
    }

    pub fn unspecified() -> UnresolvedType {
        UnresolvedType { typ: UnresolvedTypeData::Unspecified, span: None }
    }

    pub(crate) fn is_type_expression(&self) -> bool {
        matches!(&self.typ, UnresolvedTypeData::Expression(_))
    }
}

impl UnresolvedTypeData {
    pub fn from_int_token(
        token: IntType,
    ) -> Result<UnresolvedTypeData, InvalidIntegerBitSizeError> {
        use {IntType::*, UnresolvedTypeData::Integer};
        match token {
            Signed(num_bits) => {
                Ok(Integer(Signedness::Signed, IntegerBitSize::try_from(num_bits)?))
            }
            Unsigned(num_bits) => {
                Ok(Integer(Signedness::Unsigned, IntegerBitSize::try_from(num_bits)?))
            }
        }
    }

    pub fn with_span(&self, span: Span) -> UnresolvedType {
        UnresolvedType { typ: self.clone(), span: Some(span) }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, PartialOrd, Ord)]
pub enum Signedness {
    Unsigned,
    Signed,
}

impl Signedness {
    pub fn is_signed(&self) -> bool {
        match self {
            Signedness::Unsigned => false,
            Signedness::Signed => true,
        }
    }
}

impl UnresolvedTypeExpression {
    // This large error size is justified because it improves parsing speeds by around 40% in
    // release mode. See `ParserError` definition for further explanation.
    #[allow(clippy::result_large_err)]
    pub(crate) fn from_expr(
        expr: Expression,
        span: Span,
    ) -> Result<UnresolvedTypeExpression, ParserError> {
        Self::from_expr_helper(expr).map_err(|err_expr| {
            ParserError::with_reason(ParserErrorReason::InvalidTypeExpression(err_expr), span)
        })
    }

    pub fn span(&self) -> Span {
        match self {
            UnresolvedTypeExpression::Variable(path) => path.span(),
            UnresolvedTypeExpression::Constant(_, span) => *span,
            UnresolvedTypeExpression::BinaryOperation(_, _, _, span) => *span,
        }
    }

    fn from_expr_helper(expr: Expression) -> Result<UnresolvedTypeExpression, Expression> {
        match expr.kind {
            ExpressionKind::Literal(Literal::Integer(int, _)) => match int.try_to_u32() {
                Some(int) => Ok(UnresolvedTypeExpression::Constant(int, expr.span)),
                None => Err(expr),
            },
            ExpressionKind::Variable(path) => Ok(UnresolvedTypeExpression::Variable(path)),
            ExpressionKind::Prefix(prefix) if prefix.operator == UnaryOp::Minus => {
                let lhs = Box::new(UnresolvedTypeExpression::Constant(0, expr.span));
                let rhs = Box::new(UnresolvedTypeExpression::from_expr_helper(prefix.rhs)?);
                let op = BinaryTypeOperator::Subtraction;
                Ok(UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs, expr.span))
            }
            ExpressionKind::Infix(infix) if Self::operator_allowed(infix.operator.contents) => {
                let lhs = Box::new(UnresolvedTypeExpression::from_expr_helper(infix.lhs)?);
                let rhs = Box::new(UnresolvedTypeExpression::from_expr_helper(infix.rhs)?);
                let op = match infix.operator.contents {
                    BinaryOpKind::Add => BinaryTypeOperator::Addition,
                    BinaryOpKind::Subtract => BinaryTypeOperator::Subtraction,
                    BinaryOpKind::Multiply => BinaryTypeOperator::Multiplication,
                    BinaryOpKind::Divide => BinaryTypeOperator::Division,
                    BinaryOpKind::Modulo => BinaryTypeOperator::Modulo,

                    BinaryOpKind::Equal
                    | BinaryOpKind::NotEqual
                    | BinaryOpKind::Less
                    | BinaryOpKind::LessEqual
                    | BinaryOpKind::Greater
                    | BinaryOpKind::GreaterEqual
                    | BinaryOpKind::And
                    | BinaryOpKind::Or
                    | BinaryOpKind::Xor
                    | BinaryOpKind::ShiftRight
                    | BinaryOpKind::ShiftLeft => {
                        unreachable!("impossible via `operator_allowed` check")
                    }
                };
                Ok(UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs, expr.span))
            }
            _ => Err(expr),
        }
    }

    fn operator_allowed(op: BinaryOpKind) -> bool {
        matches!(
            op,
            BinaryOpKind::Add
                | BinaryOpKind::Subtract
                | BinaryOpKind::Multiply
                | BinaryOpKind::Divide
                | BinaryOpKind::Modulo
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// Represents whether the definition can be referenced outside its module/crate
pub enum ItemVisibility {
    Public,
    Private,
    PublicCrate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Represents whether the parameter is public or known only to the prover.
pub enum Visibility {
    Public,
    // Constants are not allowed in the ABI for main at the moment.
    // Constant,
    Private,
    /// DataBus is public input handled as private input. We use the fact that return values are properly computed by the program to avoid having them as public inputs
    /// it is useful for recursion and is handled by the proving system.
    /// The u32 value is used to group inputs having the same value.
    CallData(u32),
    ReturnData,
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "pub"),
            Self::Private => write!(f, "priv"),
            Self::CallData(id) => write!(f, "calldata{id}"),
            Self::ReturnData => write!(f, "returndata"),
        }
    }
}
