use std::collections::HashSet;

use noirc_frontend::field_element::FieldElement;
use nargo::errors::Location;

use arbitrary::{Arbitrary, Unstructured};
use noirc_frontend::{
    ast::{BinaryOpKind, IntegerBitSize, UnaryOp},
    monomorphization::ast::{
        ArrayLiteral, Assign, Binary, BinaryOp, Cast, Definition, Expression, FuncId, Ident,
        IdentId, If, LValue, Let, Literal, LocalId, Type, Unary,
    },
    signed_field::SignedField,
};

use super::{Name, VariableId, types, visitor::visit_expr};

/// Boolean literal.
pub fn lit_bool(value: bool) -> Expression {
    Expression::Literal(Literal::Bool(value))
}

/// Generate a literal expression according to a type.
pub fn gen_literal(u: &mut Unstructured, typ: &Type) -> arbitrary::Result<Expression> {
    use FieldElement as Field;
    use IntegerBitSize::*;

    let expr = match typ {
        Type::Unit => Expression::Literal(Literal::Unit),
        Type::Bool => lit_bool(bool::arbitrary(u)?),
        Type::U256 => {
            let field = SignedField::new(FieldElement::from(u128::arbitrary(u)?), bool::arbitrary(u)?);
            Expression::Literal(Literal::Integer(field, Type::U256, Location::dummy()))
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
                SignedField::new(field, is_negative),
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
        _ => unreachable!("unexpected type to generate a literal for: {typ}"),
    };
    Ok(expr)
}

/// Generate a literals for loop ranges with signed/unsigned integers with bits 8, 16, 32 or 64 bits,
/// in a way that start is not higher than the end, and the maximum difference between them is limited,
/// so that we don't get huge unrolled loops.
pub fn gen_range(
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
                _ => unreachable!("invalid bit size for range: {integer_bit_size} (signed)"),
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
                HundredTwentyEight => {
                    let s = u128::arbitrary(u)?;
                    let e = s.saturating_add(u.choose_index(max_size)? as u128);
                    let s = Field::from(s);
                    let e = Field::from(e);
                    (s, e)
                }
                _ => unreachable!("invalid bit size for range: {integer_bit_size} (unsigned)"),
            };
            ((s, false), (e, false))
        }
    };

    let to_lit = |(field, is_negative)| {
        Expression::Literal(Literal::Integer(
            SignedField::new(field, is_negative),
            Type::Integer(*signedness, *integer_bit_size),
            Location::dummy(),
        ))
    };

    Ok((to_lit(start), to_lit(end)))
}

/// Make an `Ident` expression out of a variable.
pub(crate) fn ident(
    variable_id: VariableId,
    id: IdentId,
    mutable: bool,
    name: Name,
    typ: Type,
) -> Expression {
    Expression::Ident(ident_inner(variable_id, id, mutable, name, typ))
}

/// Make an `Ident` out of a variable.
pub(crate) fn ident_inner(
    variable_id: VariableId,
    id: IdentId,
    mutable: bool,
    name: Name,
    typ: Type,
) -> Ident {
    Ident {
        location: None,
        definition: match variable_id {
            VariableId::Global(id) => Definition::Global(id),
            VariableId::Local(id) => Definition::Local(id),
        },
        mutable,
        name,
        typ,
        id,
    }
}

/// Integer literal, can be positive or negative depending on type.
pub fn int_literal<V>(value: V, is_negative: bool, typ: Type) -> Expression
where
    FieldElement: From<V>,
{
    Expression::Literal(Literal::Integer(
        SignedField::new(value.into(), is_negative),
        typ,
        Location::dummy(),
    ))
}

/// 8-bit unsigned int literal, used in bit shifts.
pub fn u8_literal(value: u8) -> Expression {
    int_literal(value as u32, false, types::U8)
}

/// 32-bit unsigned int literal, used in indexing arrays.
pub fn u32_literal(value: u32) -> Expression {
    int_literal(value, false, types::U32)
}

/// Create a variable.
pub fn let_var(id: LocalId, mutable: bool, name: String, expr: Expression) -> Expression {
    Expression::Let(Let { id, mutable, name, expression: Box::new(expr) })
}

/// Create an `if` expression, with an optional `else`.
pub fn if_then(
    condition: Expression,
    consequence: Expression,
    alternative: Option<Expression>,
    typ: Type,
) -> Expression {
    Expression::If(If {
        condition: Box::new(condition),
        consequence: Box::new(consequence),
        alternative: alternative.map(Box::new),
        typ,
    })
}

/// Make an if/else expression.
pub fn if_else(
    condition: Expression,
    consequence: Expression,
    alternative: Expression,
    typ: Type,
) -> Expression {
    if_then(condition, consequence, Some(alternative), typ)
}

/// Assign a value to an identifier.
pub fn assign_ident(ident: Ident, expr: Expression) -> Expression {
    Expression::Assign(Assign { lvalue: LValue::Ident(ident), expression: Box::new(expr) })
}

/// Assign a value to a mutable reference.
pub fn assign_ref(ident: Ident, expr: Expression) -> Expression {
    let typ = ident.typ.clone();
    let lvalue = LValue::Ident(ident);
    let lvalue = LValue::Dereference { reference: Box::new(lvalue), element_type: typ };
    Expression::Assign(Assign { lvalue, expression: Box::new(expr) })
}

/// Cast an expression to a target type.
pub fn cast(lhs: Expression, tgt_type: Type) -> Expression {
    Expression::Cast(Cast { lhs: Box::new(lhs), r#type: tgt_type, location: Location::dummy() })
}

/// Take an integer expression and make sure it fits in an expected `len`
/// by taking a modulo.
pub fn index_modulo(idx: Expression, len: u32) -> Expression {
    modulo(idx, u32_literal(len))
}

/// Take an integer expression and make sure it's no larger than `max_size`.
pub fn range_modulo(lhs: Expression, typ: Type, max_size: usize) -> Expression {
    modulo(lhs, int_literal(max_size as u64, false, typ))
}

/// Make a modulo expression.
pub fn modulo(lhs: Expression, rhs: Expression) -> Expression {
    binary(lhs, BinaryOpKind::Modulo, rhs)
}

/// Make an `==` expression.
pub fn equal(lhs: Expression, rhs: Expression) -> Expression {
    binary(lhs, BinaryOpKind::Equal, rhs)
}

/// Dereference an expression into a target type
pub fn deref(rhs: Expression, tgt_type: Type) -> Expression {
    unary(UnaryOp::Dereference { implicitly_added: false }, rhs, tgt_type)
}

/// Reference an expression as a target type
pub fn ref_mut(rhs: Expression, tgt_type: Type) -> Expression {
    unary(UnaryOp::Reference { mutable: true }, rhs, tgt_type)
}

/// Make a unary expression.
pub fn unary(op: UnaryOp, rhs: Expression, tgt_type: Type) -> Expression {
    Expression::Unary(Unary {
        operator: op,
        rhs: Box::new(rhs),
        result_type: tgt_type,
        location: Location::dummy(),
    })
}

/// Make a binary expression.
pub fn binary(lhs: Expression, op: BinaryOp, rhs: Expression) -> Expression {
    Expression::Binary(Binary {
        lhs: Box::new(lhs),
        operator: op,
        rhs: Box::new(rhs),
        location: Location::dummy(),
    })
}

/// Check if an `Expression` contains any `Call` in any of its descendants.
pub fn has_call(expr: &Expression) -> bool {
    exists(expr, |expr| matches!(expr, Expression::Call(_)))
}

/// Check if an `Expression` or any of its descendants match a predicate.
pub fn exists(expr: &Expression, pred: impl Fn(&Expression) -> bool) -> bool {
    let mut exists = false;
    visit_expr(expr, &mut |expr| {
        exists |= pred(expr);
        // Once we know there is a match, we can stop visiting more nodes.
        !exists
    });
    exists
}

/// Collect all the functions called in the expression and its descendants.
pub fn callees(expr: &Expression) -> HashSet<FuncId> {
    let mut callees = HashSet::default();
    visit_expr(expr, &mut |expr| {
        if let Expression::Call(call) = expr {
            if let Expression::Ident(ident) = call.func.as_ref() {
                if let Definition::Function(func_id) = ident.definition {
                    callees.insert(func_id);
                }
            }
            // Consider functions passed as arguments as at least callable.
            for arg in &call.arguments {
                if let Expression::Ident(ident) = arg {
                    if let Definition::Function(func_id) = ident.definition {
                        callees.insert(func_id);
                    }
                }
            }
        }
        true
    });
    callees
}

/// Prepend an expression to a destination.
///
/// If the destination is a `Block`, it gets prepended with a new statement,
/// otherwise it's turned into a `Block` first.
pub fn prepend(dst: &mut Expression, expr: Expression) {
    if !matches!(dst, Expression::Block(_)) {
        let mut tmp = Expression::Block(vec![]);
        std::mem::swap(dst, &mut tmp);
        let Expression::Block(stmts) = dst else {
            unreachable!("swapped with empty block");
        };
        stmts.push(tmp);
    }
    let Expression::Block(stmts) = dst else {
        unreachable!("ensured it's a block");
    };
    let mut new_stmts = vec![expr];
    new_stmts.append(stmts);
    *stmts = new_stmts;
}

/// Replace an expression with another one, passing its current value to a function.
pub fn replace(dst: &mut Expression, f: impl FnOnce(Expression) -> Expression) {
    let mut tmp = Expression::Break;
    std::mem::swap(dst, &mut tmp);
    *dst = f(tmp);
}

/// Append statements to a given block.
///
/// Panics if `block` is not `Expression::Block`.
#[allow(dead_code)]
pub fn extend_block(block: Expression, statements: Vec<Expression>) -> Expression {
    let Expression::Block(mut block_stmts) = block else {
        unreachable!("attempted to append statements to a non-block expression: {}", block)
    };

    block_stmts.extend(statements);

    Expression::Block(block_stmts)
}

/// Prepend statements to a given block.
///
/// Panics if `block` is not `Expression::Block`. Consider [prepend] which doesn't.
#[allow(dead_code)]
pub fn prepend_block(block: Expression, statements: Vec<Expression>) -> Expression {
    let Expression::Block(block_stmts) = block else {
        unreachable!("attempted to prepend statements to a non-block expression: {}", block)
    };

    let mut result_statements = vec![];
    result_statements.extend(statements);
    result_statements.extend(block_stmts);

    Expression::Block(result_statements)
}
