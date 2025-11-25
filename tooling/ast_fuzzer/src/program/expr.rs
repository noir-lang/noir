use std::collections::HashSet;

use acir::FieldElement;
use nargo::errors::Location;

use arbitrary::{Arbitrary, Unstructured};
use noirc_frontend::{
    ast::{BinaryOpKind, IntegerBitSize, UnaryOp},
    monomorphization::{
        ast::{
            ArrayLiteral, Assign, Binary, BinaryOp, Call, Cast, Definition, Expression, FuncId,
            Ident, IdentId, If, LValue, Let, Literal, LocalId, Type, Unary,
        },
        visitor::visit_expr,
    },
    signed_field::SignedField,
};

use crate::Config;

use super::{Name, VariableId, types};

/// Boolean literal.
pub fn lit_bool(value: bool) -> Expression {
    Expression::Literal(Literal::Bool(value))
}

/// Generate a literal expression according to a type.
pub fn gen_literal(
    u: &mut Unstructured,
    typ: &Type,
    config: &Config,
) -> arbitrary::Result<Expression> {
    use FieldElement as Field;
    use IntegerBitSize::*;

    let expr = match typ {
        Type::Unit => Expression::Literal(Literal::Unit),
        Type::Bool => lit_bool(bool::arbitrary(u)?),
        Type::Field => {
            let field = SignedField::new(Field::from(u128::arbitrary(u)?), bool::arbitrary(u)?);
            Expression::Literal(Literal::Integer(field, Type::Field, Location::dummy()))
        }
        Type::Integer(signedness, integer_bit_size) => {
            let (field, is_negative) = if signedness.is_signed() {
                match integer_bit_size {
                    One => bool::arbitrary(u).map(|n| (Field::from(n), n))?,
                    Eight => i8::arbitrary(u)
                        .map(|n| (Field::from(u32::from(n.unsigned_abs())), n < 0))?,
                    Sixteen => i16::arbitrary(u)
                        .map(|n| (Field::from(u32::from(n.unsigned_abs())), n < 0))?,
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
                    Eight => Field::from(u32::from(u8::arbitrary(u)?)),
                    Sixteen => Field::from(u32::from(u16::arbitrary(u)?)),
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
                arr.contents.push(gen_literal(u, item_type, config)?);
            }
            Expression::Literal(Literal::Array(arr))
        }
        Type::Slice(item_type) => {
            let len = u.int_in_range(0..=config.max_array_size)?;
            let mut arr = ArrayLiteral { contents: Vec::new(), typ: typ.clone() };
            for _ in 0..len {
                arr.contents.push(gen_literal(u, item_type, config)?);
            }
            Expression::Literal(Literal::Slice(arr))
        }
        Type::Tuple(items) => {
            let mut values = Vec::new();
            for item_type in items {
                values.push(gen_literal(u, item_type, config)?);
            }
            Expression::Tuple(values)
        }
        Type::Reference(typ, mutable) => {
            // In Noir we can return a reference for a value created in a function.
            let value = gen_literal(u, typ.as_ref(), config)?;
            ref_with_mut(value, typ.as_ref().clone(), *mutable)
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
                    let s = (Field::from(u32::from(s.unsigned_abs())), s < 0);
                    let e = (Field::from(u32::from(e.unsigned_abs())), e < 0);
                    (s, e)
                }
                Sixteen => {
                    let s = i16::arbitrary(u)?;
                    let e = s.saturating_add_unsigned(u.choose_index(max_size)? as u16);
                    let s = (Field::from(u32::from(s.unsigned_abs())), s < 0);
                    let e = (Field::from(u32::from(e.unsigned_abs())), e < 0);
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
                    let s = Field::from(u32::from(s));
                    let e = Field::from(u32::from(e));
                    (s, e)
                }
                Sixteen => {
                    let s = u16::arbitrary(u)?;
                    let e = s.saturating_add(u.choose_index(max_size)? as u16);
                    let s = Field::from(u32::from(s));
                    let e = Field::from(u32::from(e));
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
    int_literal(u32::from(value), false, types::U8)
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
    ref_with_mut(rhs, tgt_type, true)
}

fn ref_with_mut(rhs: Expression, tgt_type: Type, mutable: bool) -> Expression {
    unary(UnaryOp::Reference { mutable }, rhs, Type::Reference(Box::new(tgt_type), mutable))
}

/// Make a unary expression.
pub fn unary(op: UnaryOp, rhs: Expression, tgt_type: Type) -> Expression {
    Expression::Unary(Unary {
        operator: op,
        rhs: Box::new(rhs),
        result_type: tgt_type,
        location: Location::dummy(),
        skip: false,
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

/// Check if an `Expression` contains any `Call` another function, in any of its descendants.
/// Calls made to oracles such as `println` don't count.
pub fn has_call(expr: &Expression) -> bool {
    exists(expr, |expr| {
        let Expression::Call(Call { func, .. }) = expr else {
            return false;
        };
        // Check if we are calling an intrinsic or oracle, which don't count as recursion.
        // If we are calling a function through a reference and not an ident, then it's
        // not an oracle, so we can just assume it's recursive call.
        let Expression::Ident(Ident { definition, .. }) = func.as_ref() else {
            return true;
        };
        let is_builtin_or_oracle = matches!(
            definition,
            Definition::Builtin(_) | Definition::Oracle(_) | Definition::LowLevel(_)
        );
        !is_builtin_or_oracle
    })
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

/// Collect all the functions referred to by their ID in the expression and its descendants.
pub fn reachable_functions(expr: &Expression) -> HashSet<FuncId> {
    let mut reachable = HashSet::default();
    visit_expr(expr, &mut |expr| {
        // Regardless of whether it's in a `Call` or stored in a reference,
        // it will appear in an identifier at some point.
        if let Expression::Ident(Ident { definition: Definition::Function(func_id), .. }) = expr {
            reachable.insert(*func_id);
        }
        true
    });
    reachable
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

/// Is the expression an identifier of an immutable variable
pub(crate) fn is_immutable_ident(expr: &Expression) -> bool {
    matches!(expr, Expression::Ident(Ident { mutable: false, .. }))
}

/// Is the expression dereferencing something.
pub(crate) fn is_deref(expr: &Expression) -> bool {
    matches!(expr, Expression::Unary(Unary { operator: UnaryOp::Dereference { .. }, .. }))
}

/// Peel back any dereference operators until we get to some other kind of expression.
pub(crate) fn unref_mut(expr: &mut Expression) -> &mut Expression {
    if let Expression::Unary(Unary { operator: UnaryOp::Dereference { .. }, rhs, .. }) = expr {
        unref_mut(rhs.as_mut())
    } else {
        expr
    }
}
