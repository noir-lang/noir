use acvm::{FieldElement, acir::AcirField};
use num_bigint::BigUint;

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
    assert!(
        dfg.type_of_value(value).is_numeric(),
        "Can only cast numeric types, got {:?}",
        dfg.type_of_value(value)
    );

    if Type::Numeric(dst_typ) == dfg.type_of_value(value) {
        return SimplifiedTo(value);
    }

    if let Value::Instruction { instruction, .. } = &dfg[value] {
        if let Instruction::Cast(original_value, _) = &dfg[*instruction] {
            let original_value = *original_value;
            return match simplify_cast(original_value, dst_typ, dfg) {
                None => SimplifiedToInstruction(Instruction::Cast(original_value, dst_typ)),
                simpler => simpler,
            };
        }
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
                let integer_constant = IntegerConstant::from_numeric_constant(constant, dst_typ);
                if integer_constant.is_some() {
                    SimplifiedTo(dfg.make_constant(constant, dst_typ))
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
}
