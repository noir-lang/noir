use acvm::{acir::AcirField, FieldElement};
use num_bigint::BigUint;

use super::{DataFlowGraph, Instruction, NumericType, SimplifyResult, Type, Value, ValueId};

/// Try to simplify this cast instruction. If the instruction can be simplified to a known value,
/// that value is returned. Otherwise None is returned.
pub(super) fn simplify_cast(
    value: ValueId,
    dst_typ: NumericType,
    dfg: &mut DataFlowGraph,
) -> SimplifyResult {
    use SimplifyResult::*;
    let value = dfg.resolve(value);

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
                // Field/Unsigned -> signed
                // We only simplify to signed when we are below the maximum signed integer of the destination type.
                let integer_modulus = BigUint::from(2u128).pow(bit_size - 1);
                let constant_uint: BigUint = BigUint::from_bytes_be(&constant.to_be_bytes());
                if constant_uint < integer_modulus {
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
