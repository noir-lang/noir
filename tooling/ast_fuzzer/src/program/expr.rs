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
    let expr = match typ {
        Type::Unit => Expression::Literal(Literal::Unit),
        Type::Bool => Expression::Literal(Literal::Bool(bool::arbitrary(u)?)),
        Type::Field => {
            let field = SignedField {
                field: FieldElement::from(u128::arbitrary(u)?),
                is_negative: bool::arbitrary(u)?,
            };
            Expression::Literal(Literal::Integer(field, Type::Field, Location::dummy()))
        }
        Type::Integer(signedness, integer_bit_size) => {
            let field = match integer_bit_size {
                IntegerBitSize::One => FieldElement::from(bool::arbitrary(u)?),
                IntegerBitSize::Eight => FieldElement::from(u8::arbitrary(u)? as u32),
                IntegerBitSize::Sixteen => FieldElement::from(u16::arbitrary(u)? as u32),
                IntegerBitSize::ThirtyTwo => FieldElement::from(u32::arbitrary(u)?),
                IntegerBitSize::SixtyFour => FieldElement::from(u64::arbitrary(u)?),
                IntegerBitSize::HundredTwentyEight => FieldElement::from(u128::arbitrary(u)?),
            };

            let field =
                SignedField { field, is_negative: signedness.is_signed() && bool::arbitrary(u)? };

            Expression::Literal(Literal::Integer(
                field,
                Type::Integer(*signedness, *integer_bit_size),
                Location::dummy(),
            ))
        }
        Type::String(len) => {
            let mut s = String::new();
            for _ in 0..*len {
                s.push(char::arbitrary(u)?);
            }
            Expression::Literal(Literal::Str(s))
        }
        Type::Array(len, typ) => {
            let mut arr = ArrayLiteral { contents: Vec::new(), typ: typ.as_ref().clone() };
            for _ in 0..*len {
                arr.contents.push(gen_expr_literal(u, typ)?);
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
