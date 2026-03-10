use std::fmt::Display;

use acvm::{AcirField, FieldElement};

use crate::{
    Type,
    ast::{ExpressionKind, IntegerBitSize},
    hir_def::expr::{HirExpression, HirLiteral},
    shared::Signedness,
    token::{IntegerTypeSuffix, Token},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Integer {
    Field(FieldElement),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U1(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}

impl Integer {
    /// Convert this [Integer] to a field, returning `None` for negative values.
    /// Returns `None` for negative integers.
    pub(crate) fn as_non_negative_field(&self) -> Option<FieldElement> {
        match self {
            Integer::Field(value) => Some(*value),
            Integer::I8(value) if *value >= 0 => Some((*value).into()),
            Integer::I16(value) if *value >= 0 => Some((*value).into()),
            Integer::I32(value) if *value >= 0 => Some((*value).into()),
            Integer::I64(value) if *value >= 0 => Some((*value).into()),
            Integer::U1(value) => Some((*value).into()),
            Integer::U8(value) => Some((*value).into()),
            Integer::U16(value) => Some((*value).into()),
            Integer::U32(value) => Some((*value).into()),
            Integer::U64(value) => Some((*value).into()),
            Integer::U128(value) => Some((*value).into()),
            _ => None,
        }
    }

    /// Converts this [Integer] to a [FieldElement]. Any negative values are
    /// encoded as negative fields such that `-7 == -FieldElement::from(7)`.
    /// In other words, the resulting field is not in two's complement form.
    pub(crate) fn as_field(self) -> FieldElement {
        match self {
            Integer::Field(value) => value,
            Integer::I8(value) => value.into(),
            Integer::I16(value) => value.into(),
            Integer::I32(value) => value.into(),
            Integer::I64(value) => value.into(),
            Integer::U1(value) => value.into(),
            Integer::U8(value) => value.into(),
            Integer::U16(value) => value.into(),
            Integer::U32(value) => value.into(),
            Integer::U64(value) => value.into(),
            Integer::U128(value) => value.into(),
        }
    }

    /// Converts this [Integer] to a [FieldElement]. Any negative values are
    /// encoded in two's complement such that `-x_iN == 2^N - x`.
    /// In other words, the resulting field is not in two's complement form.
    pub(crate) fn as_field_twos_complement(self) -> FieldElement {
        match self {
            Integer::Field(value) => value,
            Integer::I8(value) => (value as u8).into(),
            Integer::I16(value) => (value as u16).into(),
            Integer::I32(value) => (value as u32).into(),
            Integer::I64(value) => (value as u64).into(),
            Integer::U1(value) => value.into(),
            Integer::U8(value) => value.into(),
            Integer::U16(value) => value.into(),
            Integer::U32(value) => value.into(),
            Integer::U64(value) => value.into(),
            Integer::U128(value) => value.into(),
        }
    }

    pub fn is_negative(&self) -> bool {
        match self {
            Integer::I8(x) => *x < 0,
            Integer::I16(x) => *x < 0,
            Integer::I32(x) => *x < 0,
            Integer::I64(x) => *x < 0,
            _ => false, // Unsigned or Field types are never negative
        }
    }

    pub fn get_type(&self) -> Type {
        match self {
            Integer::Field(_) => Type::FieldElement,
            Integer::I8(_) => Type::Integer(Signedness::Signed, IntegerBitSize::Eight),
            Integer::I16(_) => Type::Integer(Signedness::Signed, IntegerBitSize::Sixteen),
            Integer::I32(_) => Type::Integer(Signedness::Signed, IntegerBitSize::ThirtyTwo),
            Integer::I64(_) => Type::Integer(Signedness::Signed, IntegerBitSize::SixtyFour),
            Integer::U1(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::One),
            Integer::U8(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::Eight),
            Integer::U16(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::Sixteen),
            Integer::U32(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo),
            Integer::U64(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::SixtyFour),
            Integer::U128(_) => {
                Type::Integer(Signedness::Unsigned, IntegerBitSize::HundredTwentyEight)
            }
        }
    }

    pub(crate) fn into_expression_kind(self) -> ExpressionKind {
        use crate::ast::Literal::Integer as Int;
        use ExpressionKind::Literal;
        match self {
            Integer::Field(value) => Literal(Int(value, Some(IntegerTypeSuffix::Field))),
            Integer::I8(value) => Literal(Int(value.into(), Some(IntegerTypeSuffix::I8))),
            Integer::I16(value) => Literal(Int(value.into(), Some(IntegerTypeSuffix::I16))),
            Integer::I32(value) => Literal(Int(value.into(), Some(IntegerTypeSuffix::I32))),
            Integer::I64(value) => Literal(Int(value.into(), Some(IntegerTypeSuffix::I64))),
            Integer::U1(value) => Literal(Int(value.into(), Some(IntegerTypeSuffix::U1))),
            Integer::U8(value) => Literal(Int(value.into(), Some(IntegerTypeSuffix::U8))),
            Integer::U16(value) => Literal(Int(value.into(), Some(IntegerTypeSuffix::U16))),
            Integer::U32(value) => Literal(Int(value.into(), Some(IntegerTypeSuffix::U32))),
            Integer::U64(value) => Literal(Int(value.into(), Some(IntegerTypeSuffix::U64))),
            Integer::U128(value) => Literal(Int(value.into(), Some(IntegerTypeSuffix::U128))),
        }
    }

    pub(crate) fn into_hir_expression(self) -> HirExpression {
        match self {
            Integer::Field(value) => HirExpression::Literal(HirLiteral::Integer(value)),
            Integer::I8(value) => HirExpression::Literal(HirLiteral::Integer(value.into())),
            Integer::I16(value) => HirExpression::Literal(HirLiteral::Integer(value.into())),
            Integer::I32(value) => HirExpression::Literal(HirLiteral::Integer(value.into())),
            Integer::I64(value) => HirExpression::Literal(HirLiteral::Integer(value.into())),
            Integer::U1(value) => HirExpression::Literal(HirLiteral::Integer(value.into())),
            Integer::U8(value) => HirExpression::Literal(HirLiteral::Integer(value.into())),
            Integer::U16(value) => HirExpression::Literal(HirLiteral::Integer(value.into())),
            Integer::U32(value) => HirExpression::Literal(HirLiteral::Integer(value.into())),
            Integer::U64(value) => HirExpression::Literal(HirLiteral::Integer(value.into())),
            Integer::U128(value) => HirExpression::Literal(HirLiteral::Integer(value.into())),
        }
    }

    pub(crate) fn into_tokens(self) -> Vec<Token> {
        match self {
            Integer::U1(bool) => {
                vec![Token::Int(bool.into(), Some(IntegerTypeSuffix::U1))]
            }
            Integer::U8(value) => {
                vec![Token::Int(value.into(), Some(IntegerTypeSuffix::U8))]
            }
            Integer::U16(value) => {
                vec![Token::Int(value.into(), Some(IntegerTypeSuffix::U16))]
            }
            Integer::U32(value) => {
                vec![Token::Int(value.into(), Some(IntegerTypeSuffix::U32))]
            }
            Integer::U64(value) => {
                vec![Token::Int(value.into(), Some(IntegerTypeSuffix::U64))]
            }
            Integer::U128(value) => {
                vec![Token::Int(value.into(), Some(IntegerTypeSuffix::U128))]
            }
            Integer::I8(value) => {
                vec![Token::Int(value.into(), Some(IntegerTypeSuffix::I8))]
            }
            Integer::I16(value) => {
                vec![Token::Int(value.into(), Some(IntegerTypeSuffix::I16))]
            }
            Integer::I32(value) => {
                vec![Token::Int(value.into(), Some(IntegerTypeSuffix::I32))]
            }
            Integer::I64(value) => {
                vec![Token::Int(value.into(), Some(IntegerTypeSuffix::I64))]
            }
            Integer::Field(value) => {
                vec![Token::Int(value, None)]
            }
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Integer::Field(field) => field.is_zero(),
            Integer::I8(value) => *value == 0,
            Integer::I16(value) => *value == 0,
            Integer::I32(value) => *value == 0,
            Integer::I64(value) => *value == 0,
            Integer::U1(value) => !value,
            Integer::U8(value) => *value == 0,
            Integer::U16(value) => *value == 0,
            Integer::U32(value) => *value == 0,
            Integer::U64(value) => *value == 0,
            Integer::U128(value) => *value == 0,
        }
    }
}

impl Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Integer::Field(value) => write!(f, "{}", value.to_short_hex()),
            Integer::I8(value) => write!(f, "{value}"),
            Integer::I16(value) => write!(f, "{value}"),
            Integer::I32(value) => write!(f, "{value}"),
            Integer::I64(value) => write!(f, "{value}"),
            Integer::U1(false) => write!(f, "0"),
            Integer::U1(true) => write!(f, "1"),
            Integer::U8(value) => write!(f, "{value}"),
            Integer::U16(value) => write!(f, "{value}"),
            Integer::U32(value) => write!(f, "{value}"),
            Integer::U64(value) => write!(f, "{value}"),
            Integer::U128(value) => write!(f, "{value}"),
        }
    }
}

// All [Integer] operations return [None] on overflow or type mismatch
impl std::ops::Add for Integer {
    type Output = Option<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::Field(lhs), Integer::Field(rhs)) => Some(Integer::Field(lhs + rhs)),
            (Integer::U1(lhs), Integer::U1(rhs)) => {
                let result = lhs as u32 + rhs as u32;
                (result != 2).then(|| Integer::U1(result != 0))
            }
            (Integer::U8(lhs), Integer::U8(rhs)) => lhs.checked_add(rhs).map(Integer::U8),
            (Integer::U16(lhs), Integer::U16(rhs)) => lhs.checked_add(rhs).map(Integer::U16),
            (Integer::U32(lhs), Integer::U32(rhs)) => lhs.checked_add(rhs).map(Integer::U32),
            (Integer::U64(lhs), Integer::U64(rhs)) => lhs.checked_add(rhs).map(Integer::U64),
            (Integer::U128(lhs), Integer::U128(rhs)) => lhs.checked_add(rhs).map(Integer::U128),
            (Integer::I8(lhs), Integer::I8(rhs)) => lhs.checked_add(rhs).map(Integer::I8),
            (Integer::I16(lhs), Integer::I16(rhs)) => lhs.checked_add(rhs).map(Integer::I16),
            (Integer::I32(lhs), Integer::I32(rhs)) => lhs.checked_add(rhs).map(Integer::I32),
            (Integer::I64(lhs), Integer::I64(rhs)) => lhs.checked_add(rhs).map(Integer::I64),
            _ => None,
        }
    }
}

impl std::ops::Sub for Integer {
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::Field(lhs), Integer::Field(rhs)) => Some(Integer::Field(lhs - rhs)),
            (Integer::U1(lhs), Integer::U1(rhs)) => {
                let result = lhs as i32 - rhs as i32;
                (result > 0).then(|| Integer::U1(result != 0))
            }
            (Integer::U8(lhs), Integer::U8(rhs)) => lhs.checked_sub(rhs).map(Integer::U8),
            (Integer::U16(lhs), Integer::U16(rhs)) => lhs.checked_sub(rhs).map(Integer::U16),
            (Integer::U32(lhs), Integer::U32(rhs)) => lhs.checked_sub(rhs).map(Integer::U32),
            (Integer::U64(lhs), Integer::U64(rhs)) => lhs.checked_sub(rhs).map(Integer::U64),
            (Integer::U128(lhs), Integer::U128(rhs)) => lhs.checked_sub(rhs).map(Integer::U128),
            (Integer::I8(lhs), Integer::I8(rhs)) => lhs.checked_sub(rhs).map(Integer::I8),
            (Integer::I16(lhs), Integer::I16(rhs)) => lhs.checked_sub(rhs).map(Integer::I16),
            (Integer::I32(lhs), Integer::I32(rhs)) => lhs.checked_sub(rhs).map(Integer::I32),
            (Integer::I64(lhs), Integer::I64(rhs)) => lhs.checked_sub(rhs).map(Integer::I64),
            _ => None,
        }
    }
}

impl std::ops::Mul for Integer {
    type Output = Option<Self>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::Field(lhs), Integer::Field(rhs)) => Some(Integer::Field(lhs * rhs)),
            (Integer::U1(lhs), Integer::U1(rhs)) => Some(Integer::U1((lhs as u32 * rhs as u32) != 0)),
            (Integer::U8(lhs), Integer::U8(rhs)) => lhs.checked_mul(rhs).map(Integer::U8),
            (Integer::U16(lhs), Integer::U16(rhs)) => lhs.checked_mul(rhs).map(Integer::U16),
            (Integer::U32(lhs), Integer::U32(rhs)) => lhs.checked_mul(rhs).map(Integer::U32),
            (Integer::U64(lhs), Integer::U64(rhs)) => lhs.checked_mul(rhs).map(Integer::U64),
            (Integer::U128(lhs), Integer::U128(rhs)) => lhs.checked_mul(rhs).map(Integer::U128),
            (Integer::I8(lhs), Integer::I8(rhs)) => lhs.checked_mul(rhs).map(Integer::I8),
            (Integer::I16(lhs), Integer::I16(rhs)) => lhs.checked_mul(rhs).map(Integer::I16),
            (Integer::I32(lhs), Integer::I32(rhs)) => lhs.checked_mul(rhs).map(Integer::I32),
            (Integer::I64(lhs), Integer::I64(rhs)) => lhs.checked_mul(rhs).map(Integer::I64),
            _ => None,
        }
    }
}

impl std::ops::Div for Integer {
    type Output = Option<Self>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::Field(lhs), Integer::Field(rhs)) => Some(Integer::Field(lhs / rhs)),
            (Integer::U1(lhs), Integer::U1(rhs)) => {
                let result = lhs as u32 + rhs as u32;
                (result != 2).then(|| Integer::U1(result != 0))
            }
            (Integer::U8(lhs), Integer::U8(rhs)) => lhs.checked_div(rhs).map(Integer::U8),
            (Integer::U16(lhs), Integer::U16(rhs)) => lhs.checked_div(rhs).map(Integer::U16),
            (Integer::U32(lhs), Integer::U32(rhs)) => lhs.checked_sub(rhs).map(Integer::U32),
            (Integer::U64(lhs), Integer::U64(rhs)) => lhs.checked_sub(rhs).map(Integer::U64),
            (Integer::U128(lhs), Integer::U128(rhs)) => lhs.checked_sub(rhs).map(Integer::U128),
            (Integer::I8(lhs), Integer::I8(rhs)) => lhs.checked_sub(rhs).map(Integer::I8),
            (Integer::I16(lhs), Integer::I16(rhs)) => lhs.checked_sub(rhs).map(Integer::I16),
            (Integer::I32(lhs), Integer::I32(rhs)) => lhs.checked_sub(rhs).map(Integer::I32),
            (Integer::I64(lhs), Integer::I64(rhs)) => lhs.checked_sub(rhs).map(Integer::I64),
            _ => None,
        }
    }
}

impl Integer {
    /// `self < rhs`
    /// Similar to the derived `impl Ord for Integer` but will return `None` when the integer
    /// variants do not match.
    pub fn lt(&self, rhs: &Self) -> Option<bool> {
        match (self, rhs) {
            (Integer::Field(lhs), Integer::Field(rhs)) => Some(lhs < rhs),
            (Integer::U1(lhs), Integer::U1(rhs)) => Some(lhs < rhs),
            (Integer::U8(lhs), Integer::U8(rhs)) => Some(lhs < rhs),
            (Integer::U16(lhs), Integer::U16(rhs)) => Some(lhs < rhs),
            (Integer::U32(lhs), Integer::U32(rhs)) => Some(lhs < rhs),
            (Integer::U64(lhs), Integer::U64(rhs)) => Some(lhs < rhs),
            (Integer::U128(lhs), Integer::U128(rhs)) => Some(lhs < rhs),
            (Integer::I8(lhs), Integer::I8(rhs)) => Some(lhs < rhs),
            (Integer::I16(lhs), Integer::I16(rhs)) => Some(lhs < rhs),
            (Integer::I32(lhs), Integer::I32(rhs)) => Some(lhs < rhs),
            (Integer::I64(lhs), Integer::I64(rhs)) => Some(lhs < rhs),
            _ => None,
        }
    }

    /// `self <= rhs`
    /// Similar to the derived `impl Ord for Integer` but will return `None` when the integer
    /// variants do not match.
    pub fn lte(&self, rhs: &Self) -> Option<bool> {
        match (self, rhs) {
            (Integer::Field(lhs), Integer::Field(rhs)) => Some(lhs <= rhs),
            (Integer::U1(lhs), Integer::U1(rhs)) => Some(lhs <= rhs),
            (Integer::U8(lhs), Integer::U8(rhs)) => Some(lhs <= rhs),
            (Integer::U16(lhs), Integer::U16(rhs)) => Some(lhs <= rhs),
            (Integer::U32(lhs), Integer::U32(rhs)) => Some(lhs <= rhs),
            (Integer::U64(lhs), Integer::U64(rhs)) => Some(lhs <= rhs),
            (Integer::U128(lhs), Integer::U128(rhs)) => Some(lhs <= rhs),
            (Integer::I8(lhs), Integer::I8(rhs)) => Some(lhs <= rhs),
            (Integer::I16(lhs), Integer::I16(rhs)) => Some(lhs <= rhs),
            (Integer::I32(lhs), Integer::I32(rhs)) => Some(lhs <= rhs),
            (Integer::I64(lhs), Integer::I64(rhs)) => Some(lhs <= rhs),
            _ => None,
        }
    }
}
