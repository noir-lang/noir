use acir::FieldElement;
use nargo::errors::Location;

use arbitrary::{Arbitrary, Unstructured};
use noirc_frontend::{
    ast::IntegerBitSize,
    monomorphization::ast::{ArrayLiteral, Expression, Literal, Type},
    signed_field::SignedField,
};

/// Generate a literal expression according to a type.
pub(crate) fn gen_expr_literal(u: &mut Unstructured, typ: &Type) -> arbitrary::Result<Expression> {
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
                            i128::arbitrary(u).map(|n| (Field::from(n.unsigned_abs()), n < 0))?
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
                let ascii_char = u.int_in_range(0x20..=0x7e).map(char::from)?;
                s.push(ascii_char);
            }
            Expression::Literal(Literal::Str(s))
        }
        Type::Array(len, item_typ) => {
            let mut arr = ArrayLiteral { contents: Vec::new(), typ: typ.clone() };
            for _ in 0..*len {
                arr.contents.push(gen_expr_literal(u, item_typ)?);
            }
            Expression::Literal(Literal::Array(arr))
        }
        Type::Tuple(items) => {
            let mut values = Vec::new();
            for typ in items {
                values.push(gen_expr_literal(u, typ)?);
            }
            Expression::Tuple(values)
        }
        _ => unreachable!("unexpected literal type: {typ}"),
    };
    Ok(expr)
}
