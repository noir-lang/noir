use num_bigint::BigInt;

use crate::ssa::ir::{
    dfg::{DataFlowGraph, simplify::SimplifyResult},
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

    if let Value::Instruction { instruction, .. } = &dfg[value] {
        if let Instruction::Cast(original_value, _) = &dfg[*instruction] {
            return SimplifiedToInstruction(Instruction::Cast(*original_value, dst_typ));
        }
    }

    if let Some(constant) = dfg.get_numeric_constant(value) {
        let src_typ = dfg.type_of_value(value).unwrap_numeric();
        match (src_typ, dst_typ) {
            (NumericType::NativeField, NumericType::NativeField) => {
                // Field -> Field: use src value
                SimplifiedTo(value)
            }
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
                let integer_modulus = BigInt::from(1) << bit_size;
                // Handles both positive and negative constants
                let truncated =
                    ((constant % &integer_modulus) + &integer_modulus) % &integer_modulus;
                SimplifiedTo(dfg.make_constant(truncated, dst_typ))
            }
            (
                NumericType::NativeField
                | NumericType::Unsigned { .. }
                | NumericType::Signed { .. },
                NumericType::Signed { bit_size },
            ) => {
                // `IntegerConstant::from_numeric_constant` cannot handle i128
                if bit_size == 128 {
                    return None;
                }
                // Field/Unsigned -> signed
                // We could only simplify to signed when we are below the maximum integer of the destination type.
                // However, we expect that overflow constraints have been generated appropriately that enforce correctness.
                let integer_constant =
                    IntegerConstant::from_numeric_constant(constant.clone(), dst_typ);
                if integer_constant.is_some() {
                    SimplifiedTo(dfg.make_constant(constant, dst_typ))
                } else {
                    None
                }
            }
        }
    } else if Type::Numeric(dst_typ) == dfg.type_of_value(value) {
        SimplifiedTo(value)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::ssa_gen::Ssa};

    #[test]
    fn unsigned_u8_to_i8_safe() {
        let src = "
        brillig(inline) predicate_pure fn main f0 {
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
        brillig(inline) predicate_pure fn main f0 {
          b0():
            return i8 135
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
}
