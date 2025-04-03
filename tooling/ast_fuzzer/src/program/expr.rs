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

/// Make an `Ident` expression out of a variable.
pub(crate) fn ident(id: VariableId, name: Name, typ: Type) -> Expression {
    Expression::Ident(Ident {
        location: None,
        definition: match id {
            VariableId::Global(id) => Definition::Global(id),
            VariableId::Local(id) => Definition::Local(id),
        },
        mutable: false,
        name,
        typ,
    })
}

/// 32-bit unsigned int literal, used in indexing arrays.
pub(crate) fn u32_literal(value: u32) -> Expression {
    Expression::Literal(Literal::Integer(
        SignedField { field: FieldElement::from(value), is_negative: false },
        types::U32,
        Location::dummy(),
    ))
}

/// Cast an expression to a target type.
pub(crate) fn cast(lhs: Expression, tgt_type: Type) -> Expression {
    Expression::Cast(Cast { lhs: Box::new(lhs), r#type: tgt_type, location: Location::dummy() })
}

/// Take an integer expression and make sure it fits in an expected `len`
/// by taking a modulo.
pub(crate) fn index_modulo(idx: Expression, len: usize) -> Expression {
    modulo(idx, u32_literal(len as u32))
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
