use acir::FieldElement;
use nargo::errors::Location;

use arbitrary::{Arbitrary, Unstructured};
use noirc_frontend::{
    ast::{BinaryOpKind, IntegerBitSize, UnaryOp},
    monomorphization::ast::{
        ArrayLiteral, Binary, BinaryOp, Cast, Definition, Expression, Ident, Literal, Type, Unary,
    },
    signed_field::SignedField,
};

use super::{Name, VariableId, types};

/// Generate a literal expression according to a type.
pub(crate) fn gen_literal(u: &mut Unstructured, typ: &Type) -> arbitrary::Result<Expression> {
    use FieldElement as Field;
    use IntegerBitSize::*;

    let expr = match typ {
        Type::Unit => Expression::Literal(Literal::Unit),
        Type::Bool => Expression::Literal(Literal::Bool(bool::arbitrary(u)?)),
        Type::Field => {
            let field = SignedField {
                field: Field::from(u128::arbitrary(u)?),
                is_negative: bool::arbitrary(u)?,
            };
            Expression::Literal(Literal::Integer(field, Type::Field, Location::dummy()))
        }
        Type::Integer(signedness, integer_bit_size) => {
            let (field, is_negative) =
                if signedness.is_signed() {
                    match integer_bit_size {
                        One => bool::arbitrary(u).map(|n| (Field::from(n), n))?,
                        Eight => i8::arbitrary(u)
                            .map(|n| (Field::from(n.unsigned_abs() as u32), n < 0))?,
                        Sixteen => i16::arbitrary(u)
                            .map(|n| (Field::from(n.unsigned_abs() as u32), n < 0))?,
                        ThirtyTwo => {
                            i32::arbitrary(u).map(|n| (Field::from(n.unsigned_abs()), n < 0))?
                        }
                        SixtyFour => {
                            i64::arbitrary(u).map(|n| (Field::from(n.unsigned_abs()), n < 0))?
                        }
                        HundredTwentyEight => {
                            // `ssa_gen::FunctionContext::checked_numeric_constant` doesn't allow negative
                            // values with 128 bits, so let's stick to the positive range.
                            i128::arbitrary(u).map(|n| (Field::from(n.abs()), false))?
                        }
                    }
                } else {
                    let f = match integer_bit_size {
                        One => Field::from(bool::arbitrary(u)?),
                        Eight => Field::from(u8::arbitrary(u)? as u32),
                        Sixteen => Field::from(u16::arbitrary(u)? as u32),
                        ThirtyTwo => Field::from(u32::arbitrary(u)?),
                        SixtyFour => Field::from(u64::arbitrary(u)?),
                        HundredTwentyEight => Field::from(u128::arbitrary(u)?),
                    };
                    (f, false)
                };

            Expression::Literal(Literal::Integer(
                SignedField { field, is_negative },
                Type::Integer(*signedness, *integer_bit_size),
                Location::dummy(),
            ))
        }
        Type::String(len) => {
            let mut s = String::new();
            for _ in 0..*len {
                // ASCII range would be 0x20..=0x7e
                let ascii_char = u.int_in_range(65..=90).map(char::from)?;
                s.push(ascii_char);
            }
            Expression::Literal(Literal::Str(s))
        }
        Type::Array(len, item_type) => {
            let mut arr = ArrayLiteral { contents: Vec::new(), typ: typ.clone() };
            for _ in 0..*len {
                arr.contents.push(gen_literal(u, item_type)?);
            }
            Expression::Literal(Literal::Array(arr))
        }
        Type::Tuple(items) => {
            let mut values = Vec::new();
            for item_type in items {
                values.push(gen_literal(u, item_type)?);
            }
            Expression::Tuple(values)
        }
        _ => unreachable!("unexpected literal type: {typ}"),
    };
    Ok(expr)
}

/// Generate a literals for loop ranges with signed/unsigned integers with bits 8, 16, 32 or 64 bits,
/// in a way that start is not higher than the end, and the maximum difference between them is limited,
/// so that we don't get huge unrolled loops.
pub(crate) fn gen_range(
    u: &mut Unstructured,
    typ: &Type,
    max_size: usize,
) -> arbitrary::Result<(Expression, Expression)> {
    use FieldElement as Field;
    use IntegerBitSize::*;

    let Type::Integer(signedness, integer_bit_size) = typ else {
        unreachable!("invalid range type: {typ}")
    };

    let (start, end) = {
        if signedness.is_signed() {
            match integer_bit_size {
                Eight => {
                    let s = i8::arbitrary(u)?;
                    let e = s.saturating_add_unsigned(u.choose_index(max_size)? as u8);
                    let s = (Field::from(s.unsigned_abs() as u32), s < 0);
                    let e = (Field::from(e.unsigned_abs() as u32), e < 0);
                    (s, e)
                }
                Sixteen => {
                    let s = i16::arbitrary(u)?;
                    let e = s.saturating_add_unsigned(u.choose_index(max_size)? as u16);
                    let s = (Field::from(s.unsigned_abs() as u32), s < 0);
                    let e = (Field::from(e.unsigned_abs() as u32), e < 0);
                    (s, e)
                }
                ThirtyTwo => {
                    let s = i32::arbitrary(u)?;
                    let e = s.saturating_add_unsigned(u.choose_index(max_size)? as u32);
                    let s = (Field::from(s.unsigned_abs()), s < 0);
                    let e = (Field::from(e.unsigned_abs()), e < 0);
                    (s, e)
                }
                SixtyFour => {
                    let s = i64::arbitrary(u)?;
                    let e = s.saturating_add_unsigned(u.choose_index(max_size)? as u64);
                    let s = (Field::from(s.unsigned_abs()), s < 0);
                    let e = (Field::from(e.unsigned_abs()), e < 0);
                    (s, e)
                }
                _ => unreachable!("invalid bit size for range: {integer_bit_size}"),
            }
        } else {
            let (s, e) = match integer_bit_size {
                Eight => {
                    let s = u8::arbitrary(u)?;
                    let e = s.saturating_add(u.choose_index(max_size)? as u8);
                    let s = Field::from(s as u32);
                    let e = Field::from(e as u32);
                    (s, e)
                }
                Sixteen => {
                    let s = u16::arbitrary(u)?;
                    let e = s.saturating_add(u.choose_index(max_size)? as u16);
                    let s = Field::from(s as u32);
                    let e = Field::from(e as u32);
                    (s, e)
                }
                ThirtyTwo => {
                    let s = u32::arbitrary(u)?;
                    let e = s.saturating_add(u.choose_index(max_size)? as u32);
                    let s = Field::from(s);
                    let e = Field::from(e);
                    (s, e)
                }
                SixtyFour => {
                    let s = u64::arbitrary(u)?;
                    let e = s.saturating_add(u.choose_index(max_size)? as u64);
                    let s = Field::from(s);
                    let e = Field::from(e);
                    (s, e)
                }
                _ => unreachable!("invalid bit size for range: {integer_bit_size}"),
            };
            ((s, false), (e, false))
        }
    };

    let to_lit = |(field, is_negative)| {
        Expression::Literal(Literal::Integer(
            SignedField { field, is_negative },
            Type::Integer(*signedness, *integer_bit_size),
            Location::dummy(),
        ))
    };

    Ok((to_lit(start), to_lit(end)))
}

/// Make an `Ident` expression out of a variable.
pub(crate) fn ident(id: VariableId, mutable: bool, name: Name, typ: Type) -> Expression {
    Expression::Ident(ident_inner(id, mutable, name, typ))
}

/// Make an `Ident` out of a variable.
pub(crate) fn ident_inner(id: VariableId, mutable: bool, name: Name, typ: Type) -> Ident {
    Ident {
        location: None,
        definition: match id {
            VariableId::Global(id) => Definition::Global(id),
            VariableId::Local(id) => Definition::Local(id),
        },
        mutable,
        name,
        typ,
    }
}

/// 32-bit unsigned int literal, used in indexing arrays.
fn positive_int_literal<V>(value: V, typ: Type) -> Expression
where
    FieldElement: From<V>,
{
    Expression::Literal(Literal::Integer(
        SignedField { field: FieldElement::from(value), is_negative: false },
        typ,
        Location::dummy(),
    ))
}

/// 32-bit unsigned int literal, used in indexing arrays.
pub(crate) fn u32_literal(value: u32) -> Expression {
    positive_int_literal(value, types::U32)
}

/// Cast an expression to a target type.
pub(crate) fn cast(lhs: Expression, tgt_type: Type) -> Expression {
    Expression::Cast(Cast { lhs: Box::new(lhs), r#type: tgt_type, location: Location::dummy() })
}

/// Take an integer expression and make sure it fits in an expected `len`
/// by taking a modulo.
pub(crate) fn index_modulo(idx: Expression, len: u32) -> Expression {
    modulo(idx, u32_literal(len))
}

/// Take an integer expression and make sure it's no larger than `max_size`.
pub(crate) fn range_modulo(lhs: Expression, typ: Type, max_size: usize) -> Expression {
    modulo(lhs, positive_int_literal(max_size as u64, typ))
}

/// Make a modulo expression.
pub(crate) fn modulo(lhs: Expression, rhs: Expression) -> Expression {
    Expression::Binary(Binary {
        lhs: Box::new(lhs),
        operator: BinaryOpKind::Modulo,
        rhs: Box::new(rhs),
        location: Location::dummy(),
    })
}

/// Dereference an expression into a target type
pub(crate) fn deref(rhs: Expression, tgt_type: Type) -> Expression {
    unary(UnaryOp::Dereference { implicitly_added: false }, rhs, tgt_type)
}

/// Make a unary expression.
pub(crate) fn unary(op: UnaryOp, rhs: Expression, tgt_type: Type) -> Expression {
    Expression::Unary(Unary {
        operator: op,
        rhs: Box::new(rhs),
        result_type: tgt_type,
        location: Location::dummy(),
    })
}

/// Make a binary expression.
pub(crate) fn binary(lhs: Expression, op: BinaryOp, rhs: Expression) -> Expression {
    Expression::Binary(Binary {
        lhs: Box::new(lhs),
        operator: op,
        rhs: Box::new(rhs),
        location: Location::dummy(),
    })
}
