use acvm::{acir::AcirField, FieldElement};
use num_bigint::BigUint;

use super::{DataFlowGraph, Instruction, NumericType, SimplifyResult, Type, Value, ValueId};

/// Try to simplify this cast instruction. If the instruction can be simplified to a known value,
/// that value is returned. Otherwise None is returned.
pub(super) fn simplify_cast(
    value: ValueId,
    dst_typ: &Type,
    dfg: &mut DataFlowGraph,
) -> SimplifyResult {
    use SimplifyResult::*;
    let value = dfg.resolve(value);

    if let Value::Instruction { instruction, .. } = &dfg[value] {
        if let Instruction::Cast(original_value, _) = &dfg[*instruction] {
            return SimplifiedToInstruction(Instruction::Cast(*original_value, dst_typ.clone()));
        }
    }

    if let Some(constant) = dfg.get_numeric_constant(value) {
        let src_typ = dfg.type_of_value(value);
        match (src_typ, dst_typ) {
            (Type::Numeric(NumericType::NativeField), Type::Numeric(NumericType::NativeField)) => {
                // Field -> Field: use src value
                SimplifiedTo(value)
            }
            (
                Type::Numeric(NumericType::Unsigned { .. }),
                Type::Numeric(NumericType::NativeField),
            ) => {
                // Unsigned -> Field: redefine same constant as Field
                SimplifiedTo(dfg.make_constant(constant, dst_typ.clone()))
            }
            (
                Type::Numeric(
                    NumericType::NativeField
                    | NumericType::Unsigned { .. }
                    | NumericType::Signed { .. },
                ),
                Type::Numeric(NumericType::Unsigned { bit_size }),
            ) => {
                // Field/Unsigned -> unsigned: truncate
                let integer_modulus = BigUint::from(2u128).pow(*bit_size);
                let constant: BigUint = BigUint::from_bytes_be(&constant.to_be_bytes());
                let truncated = constant % integer_modulus;
                let truncated = FieldElement::from_be_bytes_reduce(&truncated.to_bytes_be());
                SimplifiedTo(dfg.make_constant(truncated, dst_typ.clone()))
            }
            _ => None,
        }
    } else if *dst_typ == dfg.type_of_value(value) {
        SimplifiedTo(value)
    } else {
        None
    }
}
