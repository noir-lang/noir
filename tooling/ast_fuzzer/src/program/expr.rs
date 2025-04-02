use acir::FieldElement;
use nargo::errors::Location;

use arbitrary::{Arbitrary, Unstructured};
use noirc_frontend::{
    ast::IntegerBitSize,
    monomorphization::ast::{
        ArrayLiteral, Cast, Definition, Expression, Ident, Index, Literal, LocalId, Type,
    },
    shared::Signedness,
    signed_field::SignedField,
};

use super::{Name, VariableId};

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
                let ascii_char = u.int_in_range(0x20..=0x7e).map(char::from)?;
                s.push(ascii_char);
            }
            Expression::Literal(Literal::Str(s))
        }
        Type::Array(len, item_typ) => {
            let mut arr = ArrayLiteral { contents: Vec::new(), typ: typ.clone() };
            for _ in 0..*len {
                arr.contents.push(gen_literal(u, item_typ)?);
            }
            Expression::Literal(Literal::Array(arr))
        }
        Type::Tuple(items) => {
            let mut values = Vec::new();
            for typ in items {
                values.push(gen_literal(u, typ)?);
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

/// Generate an expression that produces a target type from a source,
/// e.g. given a source type of `[(u32, bool); 4]` and a target of `u64`
/// it might generate `my_var[2].0 as u64`.
///
/// Returns `None` if there is no way to produce the target from the source.
pub(crate) fn gen_produce(
    u: &mut Unstructured,
    src_expr: Expression,
    src_type: &Type,
    tgt_type: &Type,
) -> arbitrary::Result<Option<Expression>> {
    let cast = |lhs| {
        Expression::Cast(Cast {
            lhs: Box::new(lhs),
            r#type: tgt_type.clone(),
            location: Location::dummy(),
        })
    };

    if src_type == tgt_type {
        // Return the variable as-is.
        return Ok(Some(src_expr));
    }

    match (src_type, tgt_type) {
        (Type::Field, Type::Integer(Signedness::Unsigned, IntegerBitSize::HundredTwentyEight)) => {
            Ok(Some(cast(src_expr)))
        }
        (Type::Bool, Type::Field) => Ok(Some(cast(src_expr))),
        (Type::Integer(Signedness::Unsigned, _), Type::Field) => Ok(Some(cast(src_expr))),
        (Type::Integer(sign_from, ibs_from), Type::Integer(sign_to, ibs_to))
            if sign_from == sign_to && ibs_from.bit_size() < ibs_to.bit_size() =>
        {
            Ok(Some(cast(src_expr)))
        }
        (Type::Reference(typ, _), _) if typ.as_ref() == tgt_type => {
            Ok(Some(Expression::Clone(Box::new(src_expr))))
        }
        (Type::Array(len, item_typ), _) => {
            // We could move this entire function into the `FunctionContext`
            // and then the index could come from another call to `gen_expr`,
            // but for now just choose a random index.
            let idx = u.choose_index(*len as usize)?;
            let idx_expr = Expression::Literal(Literal::Integer(
                SignedField { field: FieldElement::from(idx), is_negative: false },
                Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo),
                Location::dummy(),
            ));
            // Access the item.
            let item_expr = Expression::Index(Index {
                collection: Box::new(src_expr),
                index: Box::new(idx_expr),
                element_type: *item_typ.clone(),
                location: Location::dummy(),
            });
            // Produce the target type from the item.
            gen_produce(u, item_expr, item_typ, tgt_type)
        }
        (Type::Tuple(items), _) => todo!(),
        (Type::Slice(_), _) => todo!(),
        _ => {
            // We have already considered the case when the two types equal.
            // Normally we would call this function knowing that source can produce the target,
            // but in case we missed a case, let's return None and let the caller fall back to
            // a different strategy. In some cases we could return a literal, but it wouldn't
            // work in the recursive case of producing a type from an array item, which needs
            // to be wrapped with an accessor.
            Ok(None)
        }
    }
}
