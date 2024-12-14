//! The submodules of this module define the various data types required to
//! represent Noir's Ast. Of particular importance are ExpressionKind and Statement
//! which can be found in expression.rs and statement.rs respectively.
//!
//! Noir's Ast is produced by the parser and taken as input to name resolution,
//! where it is converted into the Hir (defined in the hir_def module).
mod docs;
mod expression;
mod function;
mod statement;
mod structure;
mod traits;
mod type_alias;
mod visitor;

pub use visitor::AttributeTarget;
pub use visitor::Visitor;

pub use expression::*;
pub use function::*;

#[cfg(test)]
use proptest_derive::Arbitrary;

use acvm::FieldElement;
pub use docs::*;
use noirc_errors::Span;
use serde::{Deserialize, Serialize};
pub use statement::*;
pub use structure::*;
pub use traits::*;
pub use type_alias::*;

use crate::{
    node_interner::{InternedUnresolvedTypeData, QuotedTypeId},
    parser::{ParserError, ParserErrorReason},
    token::IntType,
    BinaryTypeOperator,
};
use acvm::acir::AcirField;
use iter_extended::vecmap;

#[cfg_attr(test, derive(Arbitrary))]
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
    Named(Path, GenericTypeArgs, /*is_synthesized*/ bool),

    /// A Trait as return type or parameter of function, including its generics
    TraitAsType(Path, GenericTypeArgs),

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

    // This is an interned UnresolvedTypeData during comptime code.
    // The actual UnresolvedTypeData can be retrieved with a NodeInterner.
    Interned(InternedUnresolvedTypeData),

    Unspecified, // This is for when the user declares a variable without specifying it's type
    Error,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct UnresolvedType {
    pub typ: UnresolvedTypeData,
    pub span: Span,
}

/// An argument to a generic type or trait.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum GenericTypeArg {
    /// An ordered argument, e.g. `<A, B, C>`
    Ordered(UnresolvedType),

    /// A named argument, e.g. `<A = B, C = D, E = F>`.
    /// Used for associated types.
    Named(Ident, UnresolvedType),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum GenericTypeArgKind {
    Ordered,
    Named,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Hash)]
pub struct GenericTypeArgs {
    /// Each ordered argument, e.g. `<A, B, C>`
    pub ordered_args: Vec<UnresolvedType>,

    /// All named arguments, e.g. `<A = B, C = D, E = F>`.
    /// Used for associated types.
    pub named_args: Vec<(Ident, UnresolvedType)>,

    /// The kind of each argument, in order (in case traversing the generics in order is needed)
    pub kinds: Vec<GenericTypeArgKind>,
}

impl GenericTypeArgs {
    pub fn is_empty(&self) -> bool {
        self.ordered_args.is_empty() && self.named_args.is_empty()
    }

    fn contains_unspecified(&self) -> bool {
        let ordered_args_contains_unspecified =
            self.ordered_args.iter().any(|ordered_arg| ordered_arg.contains_unspecified());
        let named_args_contains_unspecified =
            self.named_args.iter().any(|(_name, named_arg)| named_arg.contains_unspecified());
        ordered_args_contains_unspecified || named_args_contains_unspecified
    }
}

impl From<Vec<GenericTypeArg>> for GenericTypeArgs {
    fn from(args: Vec<GenericTypeArg>) -> Self {
        let mut this = GenericTypeArgs::default();
        for arg in args {
            match arg {
                GenericTypeArg::Ordered(typ) => this.ordered_args.push(typ),
                GenericTypeArg::Named(name, typ) => this.named_args.push((name, typ)),
            }
        }
        this
    }
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
    Constant(FieldElement, Span),
    BinaryOperation(
        Box<UnresolvedTypeExpression>,
        BinaryTypeOperator,
        Box<UnresolvedTypeExpression>,
        Span,
    ),
    AsTraitPath(Box<AsTraitPath>),
}

impl Recoverable for UnresolvedType {
    fn error(span: Span) -> Self {
        UnresolvedType { typ: UnresolvedTypeData::Error, span }
    }
}

impl std::fmt::Display for GenericTypeArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenericTypeArg::Ordered(typ) => typ.fmt(f),
            GenericTypeArg::Named(name, typ) => write!(f, "{name} = {typ}"),
        }
    }
}

impl std::fmt::Display for GenericTypeArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            Ok(())
        } else {
            let mut args = vecmap(&self.ordered_args, ToString::to_string).join(", ");

            if !self.ordered_args.is_empty() && !self.named_args.is_empty() {
                args += ", ";
            }

            args += &vecmap(&self.named_args, |(name, typ)| format!("{name} = {typ}")).join(", ");
            write!(f, "<{args}>")
        }
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
            Named(s, args, _) => write!(f, "{s}{args}"),
            TraitAsType(s, args) => write!(f, "impl {s}{args}"),
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
            Interned(_) => write!(f, "?Interned"),
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
            UnresolvedTypeExpression::AsTraitPath(path) => write!(f, "{path}"),
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

    pub(crate) fn is_type_expression(&self) -> bool {
        matches!(&self.typ, UnresolvedTypeData::Expression(_))
    }

    pub fn from_path(mut path: Path) -> Self {
        let span = path.span;
        let last_segment = path.segments.last_mut().unwrap();
        let generics = last_segment.generics.take();
        let generic_type_args = if let Some(generics) = generics {
            let mut kinds = Vec::with_capacity(generics.len());
            for _ in 0..generics.len() {
                kinds.push(GenericTypeArgKind::Ordered);
            }
            GenericTypeArgs { ordered_args: generics, named_args: Vec::new(), kinds }
        } else {
            GenericTypeArgs::default()
        };
        let typ = UnresolvedTypeData::Named(path, generic_type_args, true);
        UnresolvedType { typ, span }
    }

    pub(crate) fn contains_unspecified(&self) -> bool {
        self.typ.contains_unspecified()
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
        UnresolvedType { typ: self.clone(), span }
    }

    fn contains_unspecified(&self) -> bool {
        match self {
            UnresolvedTypeData::Array(typ, length) => {
                typ.contains_unspecified() || length.contains_unspecified()
            }
            UnresolvedTypeData::Slice(typ) => typ.contains_unspecified(),
            UnresolvedTypeData::Expression(expr) => expr.contains_unspecified(),
            UnresolvedTypeData::String(length) => length.contains_unspecified(),
            UnresolvedTypeData::FormatString(typ, length) => {
                typ.contains_unspecified() || length.contains_unspecified()
            }
            UnresolvedTypeData::Parenthesized(typ) => typ.contains_unspecified(),
            UnresolvedTypeData::Named(path, args, _is_synthesized) => {
                // '_' is unspecified
                let path_is_wildcard = path.is_wildcard();
                let an_arg_is_unresolved = args.contains_unspecified();
                path_is_wildcard || an_arg_is_unresolved
            }
            UnresolvedTypeData::TraitAsType(_path, args) => args.contains_unspecified(),
            UnresolvedTypeData::MutableReference(typ) => typ.contains_unspecified(),
            UnresolvedTypeData::Tuple(args) => args.iter().any(|arg| arg.contains_unspecified()),
            UnresolvedTypeData::Function(args, ret, env, _unconstrained) => {
                let args_contains_unspecified = args.iter().any(|arg| arg.contains_unspecified());
                args_contains_unspecified
                    || ret.contains_unspecified()
                    || env.contains_unspecified()
            }
            UnresolvedTypeData::Unspecified => true,

            UnresolvedTypeData::FieldElement
            | UnresolvedTypeData::Integer(_, _)
            | UnresolvedTypeData::Bool
            | UnresolvedTypeData::Unit
            | UnresolvedTypeData::Quoted(_)
            | UnresolvedTypeData::AsTraitPath(_)
            | UnresolvedTypeData::Resolved(_)
            | UnresolvedTypeData::Interned(_)
            | UnresolvedTypeData::Error => false,
        }
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
            UnresolvedTypeExpression::AsTraitPath(path) => {
                path.trait_path.span.merge(path.impl_item.span())
            }
        }
    }

    fn from_expr_helper(expr: Expression) -> Result<UnresolvedTypeExpression, Expression> {
        match expr.kind {
            ExpressionKind::Literal(Literal::Integer(int, _)) => match int.try_to_u32() {
                Some(int) => Ok(UnresolvedTypeExpression::Constant(int.into(), expr.span)),
                None => Err(expr),
            },
            ExpressionKind::Variable(path) => Ok(UnresolvedTypeExpression::Variable(path)),
            ExpressionKind::Prefix(prefix) if prefix.operator == UnaryOp::Minus => {
                let lhs =
                    Box::new(UnresolvedTypeExpression::Constant(FieldElement::zero(), expr.span));
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
            ExpressionKind::AsTraitPath(path) => {
                Ok(UnresolvedTypeExpression::AsTraitPath(Box::new(path)))
            }
            ExpressionKind::Parenthesized(expr) => Self::from_expr_helper(*expr),
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

    fn contains_unspecified(&self) -> bool {
        match self {
            // '_' is unspecified
            UnresolvedTypeExpression::Variable(path) => path.is_wildcard(),
            UnresolvedTypeExpression::BinaryOperation(lhs, _op, rhs, _span) => {
                lhs.contains_unspecified() || rhs.contains_unspecified()
            }
            UnresolvedTypeExpression::Constant(_, _) | UnresolvedTypeExpression::AsTraitPath(_) => {
                false
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// Represents whether the definition can be referenced outside its module/crate
pub enum ItemVisibility {
    Private,
    PublicCrate,
    Public,
}

impl std::fmt::Display for ItemVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemVisibility::Public => write!(f, "pub"),
            ItemVisibility::Private => Ok(()),
            ItemVisibility::PublicCrate => write!(f, "pub(crate)"),
        }
    }
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
