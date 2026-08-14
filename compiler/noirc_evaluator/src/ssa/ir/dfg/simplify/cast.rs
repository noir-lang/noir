use acvm::{FieldElement, acir::AcirField};
use num_bigint::BigUint;

use crate::ssa::ir::{
    dfg::{DataFlowGraph, simplify::SimplifyResult, simplify::bail_malformed},
    instruction::Instruction,
    integer::IntegerConstant,
    types::{NumericType, Type},
    value::{Value, ValueId},
};

/// Try to simplify this cast instruction. If the instruction can be simplified to a known value,
/// that value is returned. Otherwise None is returned.
pub(super) fn simplify_cast(
    value: ValueId,
    dst_typ: NumericType,
    dfg: &mut DataFlowGraph,
) -> SimplifyResult {
    use SimplifyResult::*;
    if !dfg.type_of_value(value).is_numeric() {
        bail_malformed!(dfg, "cast operand must be numeric, got {:?}", dfg.type_of_value(value));
    }

    if Type::Numeric(dst_typ) == *dfg.type_of_value(value) {
        return SimplifiedTo(value);
    }

    if let Value::Instruction { instruction, .. } = &dfg[value]
        && let Instruction::Cast(original_value, intermediate_typ) = &dfg[*instruction]
    {
        let original_value = *original_value;
        let original_typ = dfg.type_of_value(original_value).unwrap_numeric();
        // Constant folding can later observe truncation or extension at the intermediate type.
        // Collapse the chain only when folding the two casts is equivalent to folding one.
        if !cast_chain_is_equivalent(original_typ, *intermediate_typ, dst_typ) {
            return None;
        }
        return match simplify_cast(original_value, dst_typ, dfg) {
            None => SimplifiedToInstruction(Instruction::Cast(original_value, dst_typ)),
            simpler => simpler,
        };
    }

    if let Some(constant) = dfg.get_numeric_constant(value) {
        let src_typ = dfg.type_of_value(value).unwrap_numeric();
        match (src_typ, dst_typ) {
            (
                NumericType::Unsigned { .. } | NumericType::Signed { .. },
                NumericType::NativeField,
            ) => {
                // Unsigned/Signed -> Field: redefine same constant as Field
                SimplifiedTo(dfg.make_constant(constant, dst_typ))
            }
            (
                NumericType::NativeField
                | NumericType::Unsigned { .. }
                | NumericType::Signed { .. },
                NumericType::Unsigned { bit_size },
            ) => {
                // Field/Unsigned -> unsigned: truncate
                let integer_modulus = BigUint::from(2u128).pow(bit_size);
                let constant: BigUint = BigUint::from_bytes_be(&constant.to_be_bytes());
                let truncated = constant % integer_modulus;
                let truncated = FieldElement::from_be_bytes_reduce(&truncated.to_bytes_be());
                SimplifiedTo(dfg.make_constant(truncated, dst_typ))
            }
            (
                NumericType::NativeField | NumericType::Unsigned { .. },
                NumericType::Signed { bit_size },
            ) => {
                // `IntegerConstant::from_numeric_constant` cannot handle i128
                if bit_size == 128 {
                    return None;
                }

                // Field/Unsigned -> signed
                // We could only simplify to signed when we are below the maximum integer of the destination type.
                // However, we expect that overflow constraints have been generated appropriately that enforce correctness.
                let integer_constant = IntegerConstant::from_numeric_constant(constant, dst_typ);
                if integer_constant.is_some() {
                    SimplifiedTo(dfg.make_constant(constant, dst_typ))
                } else {
                    None
                }
            }
            (NumericType::Signed { .. }, NumericType::Signed { bit_size }) => {
                // `IntegerConstant::from_numeric_constant` cannot handle i128
                if bit_size == 128 {
                    return None;
                }

                // When going from signed to signed, we first need to interpret the constant as the signed source type,
                // and then convert it to the signed destination type.
                // For example, when going from `i8 -1` to `i16`, `i8 -1` is represented as the FieldElement 255,
                // and it would be incorrect to use `IntegerConstant::from_numeric_constant(constant, dst_typ)` as
                // that would give `i16 255` instead of the desired `i16 -1`.
                //
                // For narrowing casts we also need to truncate to the destination width and
                // sign-extend, so that e.g. `i16 256 as i8` yields `i8 0` rather than an
                // out-of-range constant.
                if let Some(src_constant) =
                    IntegerConstant::from_numeric_constant(constant, src_typ)
                {
                    let value = src_constant.apply(|v| v, |v| v as i128);
                    let truncated = match bit_size {
                        8 => i128::from(value as i8),
                        16 => i128::from(value as i16),
                        32 => i128::from(value as i32),
                        64 => i128::from(value as i64),
                        _ => unreachable!("ICE - invalid bit size {bit_size} for signed integer"),
                    };
                    let dst_constant = IntegerConstant::Signed { value: truncated, bit_size };
                    let (dst_constant, dst_typ) = dst_constant.into_numeric_constant();
                    SimplifiedTo(dfg.make_constant(dst_constant, dst_typ))
                } else {
                    None
                }
            }
            (NumericType::NativeField, NumericType::NativeField) => {
                unreachable!("This should be covered in previous if-branch")
            }
        }
    } else {
        None
    }
}

fn cast_chain_is_equivalent(
    original_typ: NumericType,
    intermediate_typ: NumericType,
    dst_typ: NumericType,
) -> bool {
    let integer = |typ| match typ {
        NumericType::Signed { bit_size } => (bit_size, true),
        NumericType::Unsigned { bit_size } => (bit_size, false),
        NumericType::NativeField => (FieldElement::max_num_bits(), false),
    };
    let (original_bit_size, original_is_signed) = integer(original_typ);
    let (intermediate_bit_size, intermediate_is_signed) = integer(intermediate_typ);
    let (dst_bit_size, dst_is_signed) = integer(dst_typ);

    if dst_bit_size <= intermediate_bit_size {
        if dst_bit_size <= original_bit_size {
            // Both paths keep the same low destination bits.
            return true;
        }

        // The intermediate and direct casts both widen the original value. Their extension bits
        // agree unless a signed original crosses exactly one signed destination boundary.
        return !original_is_signed || intermediate_is_signed == dst_is_signed;
    }

    if intermediate_bit_size < original_bit_size {
        // The intermediate cast discards bits which the wider destination would otherwise retain.
        return false;
    }

    if intermediate_bit_size == original_bit_size {
        // Retyping equal-width bits only matters when the destination sign-extends them.
        return !dst_is_signed || intermediate_is_signed == original_is_signed;
    }

    // Both casts widen. An unsigned original always zero-extends. For a signed original, the
    // intermediate and destination signedness must agree so that extension happens at both
    // boundaries or neither one.
    !original_is_signed || intermediate_is_signed == dst_is_signed
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{interpreter::value::Value, opt::CONSTANT_FOLDING_MAX_ITER, ssa_gen::Ssa},
    };

    #[test]
    fn unsigned_u8_to_i8_safe() {
        let src = "
        brillig(inline) pure fn main f0 {
          b0():
            v2 = cast u8 135 as i8
            v3 = truncate v2 to 8 bits, max_bit_size: 9
            v4 = cast v3 as u8
            v6 = lt v4, u8 128
            constrain v6 == u1 0
            return v3
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) pure fn main f0 {
          b0():
            return i8 -121
        }
        ");
    }

    #[test]
    fn unsigned_u8_to_i8_out_of_bounds() {
        let src = "
        brillig(inline) predicate_pure fn main f0 {
          b0():
            v2 = cast u8 300 as i8
            v3 = truncate v2 to 8 bits, max_bit_size: 9
            v4 = cast v3 as u8
            v6 = lt v4, u8 128
            constrain v6 == u1 0
            return v3
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        // The overflow check would fail here.
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0():
            constrain u1 1 == u1 0
            return i8 44
        }
        ");
    }

    #[test]
    fn simplifies_out_cast_to_input_type() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: i8):
            v1 = cast u128 340282366920938463463374607431768211455 as u128
            v2 = cast v0 as i8
            return
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: i8):
            return
        }
        ");
    }

    #[test]
    fn simplifies_out_casting_there_and_back() {
        // Casting from e.g. i8 to u64 used to go through sign extending to i64,
        // which itself first cast to u8, then u64 to do some arithmetic, then
        // the result was cast to i64 and back to u64.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u64, v1: u64):
            v2 = unchecked_add v0, v1
            v3 = cast v2 as i64
            v4 = cast v3 as u64
            return v4
        }
        ";

        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u64, v1: u64):
            v2 = unchecked_add v0, v1
            v3 = cast v2 as i64
            return v2
        }
        ");
    }

    #[test]
    fn simplifies_cast_from_i8_minus_1_to_i16() {
        let src = "
        acir(inline) pure fn main f0 {
          b0():
            v0 = cast i8 -1 as i16
            return v0
        }
        ";

        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @"
        acir(inline) pure fn main f0 {
          b0():
            return i16 -1
        }
        ");
    }

    #[test]
    fn simplifies_cast_from_i16_minus_1_to_i8() {
        let src = "
        acir(inline) pure fn main f0 {
          b0():
            v0 = cast i16 -1 as i8
            return v0
        }
        ";

        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @"
        acir(inline) pure fn main f0 {
          b0():
            return i8 -1
        }
        ");
    }

    #[test]
    fn simplifies_cast_from_i16_256_to_i8() {
        let src = "
        acir(inline) pure fn main f0 {
          b0():
            v0 = cast i16 256 as i8
            return v0
        }
        ";

        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) pure fn main f0 {
          b0():
            return i8 0
        }
        ");
    }

    #[test]
    fn simplifies_cast_from_i32_384_to_i8() {
        let src = "
        acir(inline) pure fn main f0 {
          b0():
            v0 = cast i32 384 as i8
            return v0
        }
        ";

        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) pure fn main f0 {
          b0():
            return i8 -128
        }
        ");
    }

    #[test]
    fn preserves_observable_cast_boundaries() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: i8, v1: u8, v2: i16):
            constrain v0 == i8 -1
            constrain v1 == u8 255
            constrain v2 == i16 256
            v3 = cast v0 as u8
            v4 = cast v3 as i16
            v5 = cast v1 as i8
            v6 = cast v5 as i16
            v7 = truncate v2 to 8 bits, max_bit_size: 16
            v8 = cast v7 as u8
            v9 = cast v8 as u16
            v10 = truncate v2 to 8 bits, max_bit_size: 16
            v11 = cast v10 as i8
            v12 = cast v11 as i16
            v13 = cast v0 as u16
            v14 = cast v13 as i16
            return v4, v6, v9, v12, v14
        }
        ";

        let ssa = Ssa::from_str_simplifying(src).unwrap();
        let ssa = ssa.fold_constants_using_constraints(CONSTANT_FOLDING_MAX_ITER);
        let result = ssa.interpret(vec![Value::i8(-1), Value::u8(255), Value::i16(256)]).unwrap();

        assert_eq!(
            result,
            vec![Value::i16(255), Value::i16(-1), Value::u16(0), Value::i16(0), Value::i16(255),]
        );
    }

    #[test]
    fn simplifies_cast_from_field_4_to_i8() {
        let src = "
        acir(inline) pure fn main f0 {
          b0():
            v0 = cast Field 4 as i8
            return v0
        }
        ";

        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @"
        acir(inline) pure fn main f0 {
          b0():
            return i8 4
        }
        ");
    }

    #[test]
    fn simplifies_cast_from_field_255_to_i8() {
        let src = "
        acir(inline) pure fn main f0 {
          b0():
            v0 = cast Field 255 as i8
            return v0
        }
        ";

        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @"
        acir(inline) pure fn main f0 {
          b0():
            return i8 -1
        }
        ");
    }
}
