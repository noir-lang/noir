//! The submodules of this module define the various data types required to
//! represent Noir's Ast. Of particular importance are ExpressionKind and Statement
//! which can be found in expression.rs and statement.rs respectively.
//!
//! Noir's Ast is produced by the parser and taken as input to name resolution,
//! where it is converted into the Hir (defined in the hir_def module).
mod docs;
mod enumeration;
mod expression;
mod function;
mod statement;
mod structure;
mod traits;
mod type_alias;
mod visitor;

use noirc_errors::Located;
use noirc_errors::Location;
pub use visitor::AttributeTarget;
pub use visitor::Visitor;

pub use expression::*;
pub use function::*;

#[cfg(test)]
use proptest_derive::Arbitrary;

pub use docs::*;
pub use enumeration::*;
use noirc_errors::Span;
pub use statement::*;
pub use structure::*;
pub use traits::*;
pub use type_alias::*;

use crate::QuotedType;
use crate::signed_field::SignedField;
use crate::token::IntegerTypeSuffix;
use crate::{
    BinaryTypeOperator,
    node_interner::{InternedUnresolvedTypeData, QuotedTypeId},
    parser::{ParserError, ParserErrorReason},
    shared::Signedness,
};

use iter_extended::vecmap;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[cfg_attr(test, derive(Arbitrary))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Ord, PartialOrd, EnumIter)]
pub enum IntegerBitSize {
    One,
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
    HundredTwentyEight,
}

impl IntegerBitSize {
    pub fn bit_size(&self) -> u8 {
        match self {
            IntegerBitSize::One => 1,
            IntegerBitSize::Eight => 8,
            IntegerBitSize::Sixteen => 16,
            IntegerBitSize::ThirtyTwo => 32,
            IntegerBitSize::SixtyFour => 64,
            IntegerBitSize::HundredTwentyEight => 128,
        }
    }
}

impl IntegerBitSize {
    pub fn allowed_sizes() -> Vec<Self> {
        IntegerBitSize::iter().collect()
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
            HundredTwentyEight => 128,
        }
    }
}

#[derive(Debug)]
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
            128 => Ok(HundredTwentyEight),
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
    Array(UnresolvedTypeExpression, Box<UnresolvedType>), // [Field; 4] = Array(4, Field)
    Slice(Box<UnresolvedType>),
    Expression(UnresolvedTypeExpression),
    Unit,

    Parenthesized(Box<UnresolvedType>),

    /// A Named UnresolvedType can be a struct type or a type variable
    Named(Path, GenericTypeArgs, /*is_synthesized*/ bool),

    /// A Trait as return type or parameter of function, including its generics
    TraitAsType(Path, GenericTypeArgs),

    /// &T and &mut T
    Reference(Box<UnresolvedType>, /*mutable*/ bool),

    // Note: Tuples have no visibility, instead each of their elements may have one.
    Tuple(Vec<UnresolvedType>),

    Function(
        /*args:*/ Vec<UnresolvedType>,
        /*ret:*/ Box<UnresolvedType>,
        /*env:*/ Box<UnresolvedType>,
        /*unconstrained:*/ bool,
    ),

    /// An "as Trait" path leading to an associated type.
    /// E.g. `<Foo as Trait>::Bar`
    AsTraitPath(Box<AsTraitPath>),

    // TODO(audit): rename UnresolvedTypeData::Resolved ?
    /// An already resolved type. These can only be parsed if they were present in the token stream
    /// as a result of being spliced into a macro's token stream input.
    Resolved(QuotedTypeId),

    // This is an interned UnresolvedTypeData during comptime code.
    // The actual UnresolvedTypeData can be retrieved with a NodeInterner.
    Interned(InternedUnresolvedTypeData),

    Error,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct UnresolvedType {
    pub typ: UnresolvedTypeData,
    pub location: Location,
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

/// The precursor to TypeExpression, this is the type that the parser allows
/// to be used in the length position of an array type. Only constant integers, variables,
/// and numeric binary operators are allowed here.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum UnresolvedTypeExpression {
    Variable(Path),
    Constant(SignedField, Option<IntegerTypeSuffix>, Location),
    BinaryOperation(
        Box<UnresolvedTypeExpression>,
        BinaryTypeOperator,
        Box<UnresolvedTypeExpression>,
        Location,
    ),
    AsTraitPath(Box<AsTraitPath>),
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
            Array(len, typ) => write!(f, "[{typ}; {len}]"),
            Slice(typ) => write!(f, "[{typ}]"),
            Named(s, args, _) => write!(f, "{s}{args}"),
            TraitAsType(s, args) => write!(f, "impl {s}{args}"),
            Tuple(elements) => {
                let elements = vecmap(elements, ToString::to_string);
                if elements.len() == 1 {
                    write!(f, "({},)", elements[0])
                } else {
                    write!(f, "({})", elements.join(", "))
                }
            }
            Expression(expression) => expression.fmt(f),
            Function(args, ret, env, unconstrained) => {
                if *unconstrained {
                    write!(f, "unconstrained ")?;
                }

                let args = vecmap(args, ToString::to_string).join(", ");

                match &env.as_ref().typ {
                    Unit => {
                        write!(f, "fn({args}) -> {ret}")
                    }
                    Tuple(env_types) => {
                        let env_types = vecmap(env_types, |arg| arg.typ.to_string()).join(", ");
                        write!(f, "fn[{env_types}]({args}) -> {ret}")
                    }
                    other => write!(f, "fn[{other}]({args}) -> {ret}"),
                }
            }
            Reference(element, false) => write!(f, "&{element}"),
            Reference(element, true) => write!(f, "&mut {element}"),
            Unit => write!(f, "()"),
            Error => write!(f, "error"),
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
            UnresolvedTypeExpression::Constant(x, None, _) => x.fmt(f),
            UnresolvedTypeExpression::Constant(x, Some(suffix), _) => write!(f, "{x}_{suffix}"),
            UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs, _) => {
                write!(f, "({lhs} {op} {rhs})")
            }
            UnresolvedTypeExpression::AsTraitPath(path) => write!(f, "{path}"),
        }
    }
}

impl UnresolvedType {
    pub(crate) fn is_type_expression(&self) -> bool {
        matches!(&self.typ, UnresolvedTypeData::Expression(_))
    }

    pub fn from_path(mut path: Path) -> Self {
        let location = path.location;
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
        UnresolvedType { typ, location }
    }
}

impl UnresolvedTypeData {
    pub fn bool(location: Location) -> Self {
        Self::named("bool".to_string(), location)
    }

    pub fn integer(signedness: Signedness, size: IntegerBitSize, location: Location) -> Self {
        let name = match signedness {
            Signedness::Signed => match size {
                IntegerBitSize::One => "i1",
                IntegerBitSize::Eight => "i8",
                IntegerBitSize::Sixteen => "i16",
                IntegerBitSize::ThirtyTwo => "i32",
                IntegerBitSize::SixtyFour => "i64",
                IntegerBitSize::HundredTwentyEight => "i128",
            },
            Signedness::Unsigned => match size {
                IntegerBitSize::One => "u1",
                IntegerBitSize::Eight => "u8",
                IntegerBitSize::Sixteen => "u16",
                IntegerBitSize::ThirtyTwo => "u32",
                IntegerBitSize::SixtyFour => "u64",
                IntegerBitSize::HundredTwentyEight => "u128",
            },
        };
        Self::named(name.to_string(), location)
    }

    pub fn field(location: Location) -> Self {
        Self::named("Field".to_string(), location)
    }

    pub fn quoted(quoted: QuotedType, location: Location) -> Self {
        Self::named(quoted.to_string(), location)
    }

    pub fn str(length: UnresolvedTypeExpression, location: Location) -> Self {
        let ident = Ident::new("str".to_string(), location);
        let path = Path::from_ident(ident);
        Self::Named(
            path,
            GenericTypeArgs {
                ordered_args: vec![UnresolvedType {
                    typ: UnresolvedTypeData::Expression(length),
                    location,
                }],
                named_args: vec![],
                kinds: vec![GenericTypeArgKind::Ordered],
            },
            false,
        )
    }

    pub fn fmtstr(
        length: UnresolvedTypeExpression,
        element: UnresolvedType,
        location: Location,
    ) -> Self {
        let ident = Ident::new("str".to_string(), location);
        let path = Path::from_ident(ident);
        Self::Named(
            path,
            GenericTypeArgs {
                ordered_args: vec![
                    UnresolvedType { typ: UnresolvedTypeData::Expression(length), location },
                    element,
                ],
                named_args: vec![],
                kinds: vec![GenericTypeArgKind::Ordered],
            },
            false,
        )
    }

    fn named(name: String, location: Location) -> Self {
        let ident = Ident::new(name, location);
        let path = Path::from_ident(ident);
        Self::Named(path, GenericTypeArgs::default(), false)
    }

    pub fn with_location(&self, location: Location) -> UnresolvedType {
        UnresolvedType { typ: self.clone(), location }
    }

    pub fn with_dummy_location(&self) -> UnresolvedType {
        self.with_location(Location::dummy())
    }

    pub(crate) fn try_into_expression(&self) -> Option<UnresolvedTypeExpression> {
        match self {
            UnresolvedTypeData::Expression(expr) => Some(expr.clone()),
            UnresolvedTypeData::Parenthesized(unresolved_type) => {
                unresolved_type.typ.try_into_expression()
            }
            UnresolvedTypeData::Named(path, generics, _)
                if path.is_ident() && generics.is_empty() =>
            {
                Some(UnresolvedTypeExpression::Variable(path.clone()))
            }
            UnresolvedTypeData::AsTraitPath(as_trait_path) => {
                Some(UnresolvedTypeExpression::AsTraitPath(as_trait_path.clone()))
            }
            _ => None,
        }
    }
}

impl UnresolvedTypeExpression {
    // This large error size is justified because it improves parsing speeds by around 40% in
    // release mode. See `ParserError` definition for further explanation.
    #[allow(clippy::result_large_err)]
    pub(crate) fn from_expr(
        expr: Expression,
        location: Location,
    ) -> Result<UnresolvedTypeExpression, ParserError> {
        Self::from_expr_helper(expr).map_err(|err_expr| {
            ParserError::with_reason(ParserErrorReason::InvalidTypeExpression(err_expr), location)
        })
    }

    pub fn location(&self) -> Location {
        match self {
            UnresolvedTypeExpression::Variable(path) => path.location,
            UnresolvedTypeExpression::Constant(_, _, location) => *location,
            UnresolvedTypeExpression::BinaryOperation(_, _, _, location) => *location,
            UnresolvedTypeExpression::AsTraitPath(path) => {
                path.trait_path.location.merge(path.impl_item.location())
            }
        }
    }

    pub fn span(&self) -> Span {
        self.location().span
    }

    fn from_expr_helper(expr: Expression) -> Result<UnresolvedTypeExpression, Expression> {
        match expr.kind {
            ExpressionKind::Literal(Literal::Integer(int, suffix)) => {
                Ok(UnresolvedTypeExpression::Constant(int, suffix, expr.location))
            }
            ExpressionKind::Variable(path) => Ok(UnresolvedTypeExpression::Variable(path)),
            ExpressionKind::Prefix(prefix) if prefix.operator == UnaryOp::Minus => {
                let lhs = Box::new(UnresolvedTypeExpression::Constant(
                    SignedField::zero(),
                    None,
                    expr.location,
                ));
                let rhs = Box::new(UnresolvedTypeExpression::from_expr_helper(prefix.rhs)?);
                let op = BinaryTypeOperator::Subtraction;
                Ok(UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs, expr.location))
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
                Ok(UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs, expr.location))
            }
            ExpressionKind::AsTraitPath(path) => Ok(UnresolvedTypeExpression::AsTraitPath(path)),
            ExpressionKind::Parenthesized(expr) => Self::from_expr_helper(*expr),
            _ => Err(expr),
        }
    }

    pub fn to_expression_kind(&self) -> ExpressionKind {
        match self {
            UnresolvedTypeExpression::Variable(path) => ExpressionKind::Variable(path.clone()),
            UnresolvedTypeExpression::Constant(int, suffix, _) => {
                ExpressionKind::Literal(Literal::Integer(*int, *suffix))
            }
            UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs, location) => {
                ExpressionKind::Infix(Box::new(InfixExpression {
                    lhs: Expression { kind: lhs.to_expression_kind(), location: *location },
                    operator: Located::from(*location, op.operator_to_binary_op_kind_helper()),
                    rhs: Expression { kind: rhs.to_expression_kind(), location: *location },
                }))
            }
            UnresolvedTypeExpression::AsTraitPath(path) => {
                ExpressionKind::AsTraitPath(Box::new(*path.clone()))
            }
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

    pub(crate) fn is_valid_expression(&self) -> bool {
        match self {
            UnresolvedTypeExpression::Variable(path) => path.no_generic(),
            UnresolvedTypeExpression::Constant(_, _, _) => true,
            UnresolvedTypeExpression::BinaryOperation(lhs, _, rhs, _) => {
                lhs.is_valid_expression() && rhs.is_valid_expression()
            }
            UnresolvedTypeExpression::AsTraitPath(_) => true,
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

impl ItemVisibility {
    pub(crate) fn is_private(&self) -> bool {
        matches!(self, ItemVisibility::Private)
    }
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
