use std::collections::HashSet;

use acir::FieldElement;
use iter_extended::vecmap;
use noirc_frontend::{
    ast::{BinaryOpKind, IntegerBitSize},
    hir_def,
    monomorphization::ast::{BinaryOp, Type},
    shared::Signedness,
};
use strum::IntoEnumIterator as _;

pub const U8: Type = Type::Integer(Signedness::Unsigned, IntegerBitSize::Eight);
pub const U32: Type = Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo);

/// Calculate the depth of a type.
///
/// Leaf types have a depth of 0.
pub fn type_depth(typ: &Type) -> usize {
    match typ {
        Type::Field | Type::Bool | Type::String(_) | Type::Unit | Type::Integer(_, _) => 0,
        Type::Array(_, typ) => 1 + type_depth(typ),
        Type::Tuple(types) => 1 + types.iter().map(type_depth).max().unwrap_or_default(),
        _ => unreachable!("unexpected type: {typ}"),
    }
}

/// We can only use globals that can be evaluated at comptime.
/// Some types don't compile in Noir, so avoid those as we couldn't
/// put any related failures into an integration test.
pub fn can_be_global(typ: &Type) -> bool {
    !matches!(
        typ,
        Type::Integer(Signedness::Signed, IntegerBitSize::One | IntegerBitSize::HundredTwentyEight)
            | Type::Integer(Signedness::Unsigned, IntegerBitSize::One)
    )
}

/// Check if a type can be used in the `main` function.
///
/// We decided we will avoid 0 length arrays in the main inputs and outputs, because we don't generate
/// witnesses for them anyway, and they are tricky to handle consistently when they can be regular inputs
/// as well as part of the databus. They are not expected in real programs as they don't do anything useful.
pub fn can_be_main(typ: &Type) -> bool {
    match typ {
        Type::String(size) => *size > 0,
        Type::Array(size, typ) => *size > 0 && can_be_main(typ),
        Type::Tuple(types) => types.iter().all(can_be_main),
        Type::Bool | Type::Field | Type::Integer(_, _) => true,
        _ => false,
    }
}

/// Collect all the sub-types produced by a type.
///
/// It's like a _power set_ of the type.
pub fn types_produced(typ: &Type) -> HashSet<Type> {
    /// Recursively visit subtypes.
    fn visit(acc: &mut HashSet<Type>, typ: &Type) {
        if acc.contains(typ) {
            return;
        }

        // Trivially produce self.
        acc.insert(typ.clone());

        match typ {
            Type::Array(len, typ) => {
                if *len > 0 {
                    visit(acc, typ);
                }
                // Technically we could produce `[T; N]` from `[S; N]` if
                // we can produce `T` from `S`, but let's ignore that;
                // instead we will produce `[T; N]` from any source that can
                // supply `T`, one of which would be the `[S; N]` itself.
                // So if we have `let foo = [1u32, 2u32];` and we need `[u64; 2]`
                // we might generate `[foo[1] as u64, 3u64]` instead of "mapping"
                // over the entire foo. Same goes for tuples.
            }
            Type::Tuple(types) => {
                for typ in types {
                    visit(acc, typ);
                }
            }
            Type::String(_) => {
                // Maybe it could produce substrings, but it would be an overkill to enumerate.
            }
            Type::Field => {
                // There are `try_to_*` methods, but let's consider only what is safe.
                acc.insert(Type::Integer(Signedness::Unsigned, IntegerBitSize::HundredTwentyEight));
            }
            Type::Integer(sign, integer_bit_size) => {
                // Casting up is safe.
                for size in IntegerBitSize::iter()
                    .filter(|size| size.bit_size() > integer_bit_size.bit_size())
                {
                    // We don't want to produce `i1` or `i128`
                    if sign.is_signed()
                        && (size == IntegerBitSize::One
                            || size == IntegerBitSize::HundredTwentyEight)
                    {
                        continue;
                    }

                    acc.insert(Type::Integer(*sign, size));
                }
                // There are `From<u*>` for Field
                if !sign.is_signed() {
                    acc.insert(Type::Field);
                }
            }
            Type::Bool => {
                // Maybe we can also cast to u1 or u8 etc?
                acc.insert(Type::Field);
            }
            Type::Slice(typ) => {
                visit(acc, typ);
            }
            Type::Reference(typ, _) => {
                visit(acc, typ);
            }
            Type::Function(_, ret, _, _) => {
                visit(acc, ret);
            }
            Type::Unit | Type::FmtString(_, _) => {}
        }
    }

    let mut acc = HashSet::new();
    visit(&mut acc, typ);
    acc
}

pub fn to_hir_type(typ: &Type) -> hir_def::types::Type {
    use hir_def::types::{Kind as HirKind, Type as HirType};

    // Meet the expectations of `Type::evaluate_to_u32`.
    let size_const = |size: u32| {
        Box::new(HirType::Constant(
            FieldElement::from(size),
            HirKind::Numeric(Box::new(HirType::Integer(
                Signedness::Unsigned,
                IntegerBitSize::ThirtyTwo,
            ))),
        ))
    };

    match typ {
        Type::Unit => HirType::Unit,
        Type::Bool => HirType::Bool,
        Type::Field => HirType::FieldElement,
        Type::Integer(signedness, integer_bit_size) => {
            HirType::Integer(*signedness, *integer_bit_size)
        }
        Type::String(size) => HirType::String(size_const(*size)),
        Type::Array(size, typ) => HirType::Array(size_const(*size), Box::new(to_hir_type(typ))),
        Type::Tuple(items) => HirType::Tuple(items.iter().map(to_hir_type).collect()),
        Type::Function(param_types, return_type, env_type, unconstrained) => HirType::Function(
            vecmap(param_types, to_hir_type),
            Box::new(to_hir_type(return_type)),
            Box::new(to_hir_type(env_type)),
            *unconstrained,
        ),
        Type::Reference(typ, mutable) => HirType::Reference(Box::new(to_hir_type(typ)), *mutable),
        Type::FmtString(_, _) | Type::Slice(_) => {
            unreachable!("unexpected type converting to HIR: {}", typ)
        }
    }
}

/// Check if the type is a number.
pub fn is_numeric(typ: &Type) -> bool {
    matches!(typ, Type::Field | Type::Integer(_, _))
}

/// Check if a type is `Unit`.
pub fn is_unit(typ: &Type) -> bool {
    matches!(typ, Type::Unit)
}

/// Check if the type works with `UnaryOp::Not`
pub fn is_bool(typ: &Type) -> bool {
    matches!(typ, Type::Bool)
}

/// Check if the type is a reference wrapping another.
pub fn is_reference(typ: &Type) -> bool {
    matches!(typ, Type::Reference(_, _))
}

/// Check if the type can be used with a `println` statement.
pub fn is_printable(typ: &Type) -> bool {
    match typ {
        Type::Reference(_, _) => false,
        Type::Field | Type::Integer(_, _) | Type::String(_) | Type::Bool | Type::Unit => true,
        Type::Slice(typ) | Type::Array(_, typ) | Type::FmtString(_, typ) => is_printable(typ),
        Type::Tuple(typs) => typs.iter().all(is_printable),
        // Function signatures are printable, although they might differ when we force things to be Brillig.
        Type::Function(_, _, _, _) => true,
    }
}

/// Can the type be returned by some `UnaryOp`.
pub fn can_unary_return(typ: &Type) -> bool {
    match typ {
        Type::Field => true,
        Type::Bool => true,
        Type::Integer(sign, size) => {
            // What can we apply `UnaryOp::Minus` to.
            // The number has to be signed, otherwise it doesn't have a negative.
            sign.is_signed() &&
            // i1 range is -1..0, so unless it's 0 it will fail
            size.bit_size() > 1 &&
            // i128 is not a type the user can declare, but trying to use minus with it
            // would involve a truncation to 129 bits, which wants to convert to u128,
            // and 2**129 wouldn't fit into that, and we end up with a division by zero
            size.bit_size() < 128
        }
        _ => false,
    }
}

/// Can the type be returned by some `BinaryOp`.
pub fn can_binary_return(typ: &Type) -> bool {
    BinaryOp::iter().any(|op| can_binary_op_return(&op, typ))
}

/// Check if a certain binary operation can return a target type as output.
pub fn can_binary_op_return(op: &BinaryOp, typ: &Type) -> bool {
    use BinaryOpKind::*;
    match typ {
        Type::Bool => op.is_comparator(),
        Type::Field => {
            matches!(op, Add | Subtract | Multiply | Divide)
        }
        Type::Integer(_, _) => {
            matches!(op, Add | Subtract | Multiply | Divide | ShiftLeft | ShiftRight | Modulo)
        }
        _ => false,
    }
}

/// Can the binary operation result in an overflow.
/// These are operations that commonly fail with random constants.
pub fn can_binary_op_overflow(op: &BinaryOp) -> bool {
    use BinaryOpKind::*;
    matches!(op, Add | Subtract | Multiply | ShiftLeft)
}

/// Can the binary operation fail because the RHS is zero.
/// These operations can fail because of an unfortunate combination of
/// expressions, such as `a / (a % a)` or `a % (a % a)`, but it's less
/// like to occur than the overflows.
pub fn can_binary_op_err_by_zero(op: &BinaryOp) -> bool {
    use BinaryOpKind::*;
    matches!(op, Divide | Modulo)
}

/// Check if a certain binary operation can take a type as input and produce the target output.
pub fn can_binary_op_return_from_input(op: &BinaryOp, input: &Type, output: &Type) -> bool {
    match (input, output) {
        (Type::Field, Type::Field) => op.is_valid_for_field_type() && !op.is_equality(),
        (Type::Field, Type::Bool) => op.is_equality(),
        (Type::Bool, Type::Bool) => op.is_comparator() || op.is_bitwise(),
        (Type::Integer(sign, size), Type::Bool) => {
            // Avoid comparing 128 bit numbers:
            // `AcirContext::less_than_signed` would cause overflow with i128
            // `AcirContext::euclidean_division_var` would divide by zero with u128
            op.is_comparator() && (op.is_equality() || size.bit_size() != 128 || !sign.is_signed())
        }
        (Type::Integer(sign_in, size_in), Type::Integer(sign_out, size_out))
            if sign_in == sign_out =>
        {
            let size = size_in.bit_size();
            // i1 and u1 are very easy to overflow, so we might want to disable those, to not get trivial assertion errors.
            // i128 is not a type a user can define, and the truncation that gets added after binary operations to
            // limit it to 129 bits results in division by zero during compilation.
            (op.is_arithmetic() && size != 1 && size != 128 && size_in <= size_out)
                || op.is_bitshift() && size != 128 && !(size == 1 && sign_in.is_signed())
                || op.is_bitwise()
        }
        (Type::Reference(typ, _), _) => can_binary_op_return_from_input(op, typ, output),
        _ => false,
    }
}

/// Reference an expression into a target type
pub fn ref_mut(typ: Type) -> Type {
    Type::Reference(Box::new(typ), true)
}
