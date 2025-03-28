use acvm::{AcirField as _, FieldElement};
use binary::simplify_binary;
use call::simplify_call;
use cast::simplify_cast;
use constrain::decompose_constrain;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        call_stack::CallStackId,
        instruction::{
            Binary, BinaryOp, Instruction,
            binary::{truncate, truncate_field},
        },
        types::Type,
        value::{Value, ValueId},
    },
    opt::flatten_cfg::value_merger::ValueMerger,
};

use super::DataFlowGraph;

mod binary;
mod call;
mod cast;
mod constrain;

/// Contains the result to Instruction::simplify, specifying how the instruction
/// should be simplified.
pub(crate) enum SimplifyResult {
    /// Replace this function's result with the given value
    SimplifiedTo(ValueId),

    /// Replace this function's results with the given values
    /// Used for when there are multiple return values from
    /// a function such as a tuple
    SimplifiedToMultiple(Vec<ValueId>),

    /// Replace this function with an simpler but equivalent instruction.
    SimplifiedToInstruction(Instruction),

    /// Replace this function with a set of simpler but equivalent instructions.
    /// This is currently only to be used for [`Instruction::Constrain`].
    SimplifiedToInstructionMultiple(Vec<Instruction>),

    /// Remove the instruction, it is unnecessary
    Remove,

    /// Instruction could not be simplified
    None,
}

impl SimplifyResult {
    pub(crate) fn instructions(self) -> Option<Vec<Instruction>> {
        match self {
            SimplifyResult::SimplifiedToInstruction(instruction) => Some(vec![instruction]),
            SimplifyResult::SimplifiedToInstructionMultiple(instructions) => Some(instructions),
            _ => None,
        }
    }
}

/// Try to simplify this instruction. If the instruction can be simplified to a known value,
/// that value is returned. Otherwise None is returned.
///
/// The `block` parameter indicates the block this new instruction will be inserted into
/// after this call.
pub(crate) fn simplify(
    instruction: &Instruction,
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    ctrl_typevars: Option<Vec<Type>>,
    call_stack: CallStackId,
) -> SimplifyResult {
    use SimplifyResult::*;

    match instruction {
        Instruction::Binary(binary) => simplify_binary(binary, dfg),
        Instruction::Cast(value, typ) => simplify_cast(*value, *typ, dfg),
        Instruction::Not(value) => {
            match &dfg[dfg.resolve(*value)] {
                // Limit optimizing ! on constants to only booleans. If we tried it on fields,
                // there is no Not on FieldElement, so we'd need to convert between u128. This
                // would be incorrect however since the extra bits on the field would not be flipped.
                Value::NumericConstant { constant, typ } if typ.is_unsigned() => {
                    // As we're casting to a `u128`, we need to clear out any upper bits that the NOT fills.
                    let bit_size = typ.bit_size();
                    assert!(bit_size <= 128);
                    let not_value: u128 = truncate(!constant.to_u128(), bit_size);
                    SimplifiedTo(dfg.make_constant(not_value.into(), *typ))
                }
                Value::Instruction { instruction, .. } => {
                    // !!v => v
                    if let Instruction::Not(value) = &dfg[*instruction] {
                        SimplifiedTo(*value)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        Instruction::Constrain(lhs, rhs, msg) => {
            let constraints = decompose_constrain(*lhs, *rhs, msg, dfg);
            if constraints.is_empty() {
                Remove
            } else {
                SimplifiedToInstructionMultiple(constraints)
            }
        }
        Instruction::ConstrainNotEqual(..) => None,
        Instruction::ArrayGet { array, index } => {
            if let Some(index) = dfg.get_numeric_constant(*index) {
                try_optimize_array_get_from_previous_set(dfg, *array, index)
            } else {
                None
            }
        }
        Instruction::ArraySet { array: array_id, index: index_id, value, .. } => {
            let array = dfg.get_array_constant(*array_id);
            let index = dfg.get_numeric_constant(*index_id);
            if let (Some((array, _element_type)), Some(index)) = (array, index) {
                let index =
                    index.try_to_u32().expect("Expected array index to fit in u32") as usize;

                if index < array.len() {
                    let elements = array.update(index, *value);
                    let typ = dfg.type_of_value(*array_id);
                    let instruction = Instruction::MakeArray { elements, typ };
                    let new_array = dfg.insert_instruction_and_results(
                        instruction,
                        block,
                        Option::None,
                        call_stack,
                    );
                    return SimplifiedTo(new_array.first());
                }
            }

            try_optimize_array_set_from_previous_get(dfg, *array_id, *index_id, *value)
        }
        Instruction::Truncate { value, bit_size, max_bit_size } => {
            if bit_size == max_bit_size {
                return SimplifiedTo(*value);
            }
            if let Some((numeric_constant, typ)) = dfg.get_numeric_constant_with_type(*value) {
                let truncated_field = truncate_field(numeric_constant, *bit_size);
                SimplifiedTo(dfg.make_constant(truncated_field, typ))
            } else if let Value::Instruction { instruction, .. } = &dfg[dfg.resolve(*value)] {
                match &dfg[*instruction] {
                    Instruction::Truncate { bit_size: src_bit_size, .. } => {
                        // If we're truncating the value to fit into the same or larger bit size then this is a noop.
                        if src_bit_size <= bit_size && src_bit_size <= max_bit_size {
                            SimplifiedTo(*value)
                        } else {
                            None
                        }
                    }

                    Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Div, .. })
                        if dfg.is_constant(*rhs) =>
                    {
                        // If we're truncating the result of a division by a constant denominator, we can
                        // reason about the maximum bit size of the result and whether a truncation is necessary.

                        let numerator_type = dfg.type_of_value(*lhs);
                        let max_numerator_bits = numerator_type.bit_size();

                        let divisor =
                            dfg.get_numeric_constant(*rhs).expect("rhs is checked to be constant.");
                        let divisor_bits = divisor.num_bits();

                        // 2^{max_quotient_bits} = 2^{max_numerator_bits} / 2^{divisor_bits}
                        // => max_quotient_bits = max_numerator_bits - divisor_bits
                        //
                        // In order for the truncation to be a noop, we then require `max_quotient_bits < bit_size`.
                        let max_quotient_bits = max_numerator_bits - divisor_bits;
                        if max_quotient_bits < *bit_size { SimplifiedTo(*value) } else { None }
                    }

                    _ => None,
                }
            } else {
                None
            }
        }
        Instruction::Call { func, arguments } => {
            simplify_call(*func, arguments, dfg, block, ctrl_typevars, call_stack)
        }
        Instruction::EnableSideEffectsIf { condition } => {
            if let Some(last) = dfg[block].instructions().last().copied() {
                let last = &mut dfg[last];
                if matches!(last, Instruction::EnableSideEffectsIf { .. }) {
                    *last = Instruction::EnableSideEffectsIf { condition: *condition };
                    return Remove;
                }
            }
            None
        }
        Instruction::Allocate { .. } => None,
        Instruction::Load { .. } => None,
        Instruction::Store { .. } => None,
        Instruction::IncrementRc { .. } => None,
        Instruction::DecrementRc { .. } => None,
        Instruction::RangeCheck { value, max_bit_size, .. } => {
            let max_potential_bits = dfg.get_value_max_num_bits(*value);
            if max_potential_bits <= *max_bit_size { Remove } else { None }
        }
        Instruction::IfElse { then_condition, then_value, else_condition, else_value } => {
            let then_condition = dfg.resolve(*then_condition);
            let else_condition = dfg.resolve(*else_condition);
            let typ = dfg.type_of_value(*then_value);

            if let Some(constant) = dfg.get_numeric_constant(then_condition) {
                if constant.is_one() {
                    return SimplifiedTo(*then_value);
                } else if constant.is_zero() {
                    return SimplifiedTo(*else_value);
                }
            }

            let then_value = dfg.resolve(*then_value);
            let else_value = dfg.resolve(*else_value);
            if then_value == else_value {
                return SimplifiedTo(then_value);
            }

            if let Value::Instruction { instruction, .. } = &dfg[then_value] {
                if let Instruction::IfElse {
                    then_condition: inner_then_condition,
                    then_value: inner_then_value,
                    else_condition: inner_else_condition,
                    ..
                } = dfg[*instruction]
                {
                    if then_condition == inner_then_condition {
                        let instruction = Instruction::IfElse {
                            then_condition,
                            then_value: inner_then_value,
                            else_condition: inner_else_condition,
                            else_value,
                        };
                        return SimplifiedToInstruction(instruction);
                    }
                    // TODO: We could check to see if `then_condition == inner_else_condition`
                    // but we run into issues with duplicate NOT instructions having distinct ValueIds.
                }
            };

            if let Value::Instruction { instruction, .. } = &dfg[else_value] {
                if let Instruction::IfElse {
                    then_condition: inner_then_condition,
                    else_condition: inner_else_condition,
                    else_value: inner_else_value,
                    ..
                } = dfg[*instruction]
                {
                    if then_condition == inner_then_condition {
                        let instruction = Instruction::IfElse {
                            then_condition,
                            then_value,
                            else_condition: inner_else_condition,
                            else_value: inner_else_value,
                        };
                        return SimplifiedToInstruction(instruction);
                    }
                    // TODO: We could check to see if `then_condition == inner_else_condition`
                    // but we run into issues with duplicate NOT instructions having distinct ValueIds.
                }
            };

            if matches!(&typ, Type::Numeric(_)) {
                let result = ValueMerger::merge_numeric_values(
                    dfg,
                    block,
                    then_condition,
                    else_condition,
                    then_value,
                    else_value,
                );
                SimplifiedTo(result)
            } else {
                None
            }
        }
        Instruction::MakeArray { .. } => None,
        Instruction::Noop => Remove,
    }
}

/// Given a chain of operations like:
/// v1 = array_set [10, 11, 12], index 1, value: 5
/// v2 = array_set v1, index 2, value: 6
/// v3 = array_set v2, index 2, value: 7
/// v4 = array_get v3, index 1
///
/// We want to optimize `v4` to `10`. To do this we need to follow the array value
/// through several array sets. For each array set:
/// - If the index is non-constant we fail the optimization since any index may be changed
/// - If the index is constant and is our target index, we conservatively fail the optimization
///   in case the array_set is disabled from a previous `enable_side_effects_if` and the array get
///   was not.
/// - Otherwise, we check the array value of the array set.
///   - If the array value is constant, we use that array.
///   - If the array value is from a previous array-set, we recur.
fn try_optimize_array_get_from_previous_set(
    dfg: &DataFlowGraph,
    mut array_id: ValueId,
    target_index: FieldElement,
) -> SimplifyResult {
    let mut elements = None;

    // Arbitrary number of maximum tries just to prevent this optimization from taking too long.
    let max_tries = 5;
    for _ in 0..max_tries {
        if let Some(instruction) = dfg.get_local_or_global_instruction(array_id) {
            match instruction {
                Instruction::ArraySet { array, index, value, .. } => {
                    if let Some(constant) = dfg.get_numeric_constant(*index) {
                        if constant == target_index {
                            return SimplifyResult::SimplifiedTo(*value);
                        }

                        array_id = *array; // recur
                    } else {
                        return SimplifyResult::None;
                    }
                }
                Instruction::MakeArray { elements: array, typ: _ } => {
                    elements = Some(array.clone());
                    break;
                }
                _ => return SimplifyResult::None,
            }
        } else {
            return SimplifyResult::None;
        }
    }

    if let (Some(array), Some(index)) = (elements, target_index.try_to_u64()) {
        let index = index as usize;
        if index < array.len() {
            return SimplifyResult::SimplifiedTo(array[index]);
        }
    }
    SimplifyResult::None
}

/// If we have an array set whose value is from an array get on the same array at the same index,
/// we can simplify that array set to the array we were looking to perform an array set upon.
///
/// Simple case:
/// v3 = array_get v1, index v2
/// v5 = array_set v1, index v2, value v3
///
/// If we could not immediately simplify the array set from its value, we can try to follow
/// the array set backwards in the case we have constant indices:
///
/// v3 = array_get v1, index 1
/// v5 = array_set v1, index 2, value [Field 100, Field 101, Field 102]
/// v7 = array_set mut v5, index 1, value v3
///
/// We want to optimize `v7` to `v5`. We see that `v3` comes from an array get to `v1`. We follow `v5` backwards and see an array set
/// to `v1` and see that the previous array set occurs to a different constant index.
///
/// For each array_set:
/// - If the index is non-constant we fail the optimization since any index may be changed.
/// - If the index is constant and is our target index, we conservatively fail the optimization.
/// - Otherwise, we check the array value of the `array_set`. We will refer to this array as array'.
///   In the case above, array' is `v1` from `v5 = array set ...`
///   - If the original `array_set` value comes from an `array_get`, check the array in that `array_get` against array'.
///   - If the two values are equal we can simplify.
///     - Continuing the example above, as we have `v3 = array_get v1, index 1`, `v1` is
///       what we want to check against array'. We now know we can simplify `v7` to `v5` as it is unchanged.
///   - If they are not equal, recur marking the current `array_set` array as the new array id to use in the checks
fn try_optimize_array_set_from_previous_get(
    dfg: &DataFlowGraph,
    mut array_id: ValueId,
    target_index: ValueId,
    target_value: ValueId,
) -> SimplifyResult {
    let array_from_get = match &dfg[target_value] {
        Value::Instruction { instruction, .. } => match &dfg[*instruction] {
            Instruction::ArrayGet { array, index } => {
                if *array == array_id && *index == target_index {
                    // If array and index match from the value, we can immediately simplify
                    return SimplifyResult::SimplifiedTo(array_id);
                } else if *index == target_index {
                    *array
                } else {
                    return SimplifyResult::None;
                }
            }
            _ => return SimplifyResult::None,
        },
        _ => return SimplifyResult::None,
    };

    // At this point we have determined that the value we are writing in the `array_set` instruction
    // comes from an `array_get` from the same index at which we want to write it at.
    // It's possible that we're acting on the same array where other indices have been mutated in between
    // the `array_get` and `array_set` (resulting in the `array_id` not matching).
    //
    // We then inspect the set of `array_set`s which which led to the current array the `array_set` is acting on.
    // If we can work back to the array on which the `array_get` was reading from without having another `array_set`
    // act on the same index then we can be sure that the new `array_set` can be removed without affecting the final result.
    let Some(target_index) = dfg.get_numeric_constant(target_index) else {
        return SimplifyResult::None;
    };

    let original_array_id = array_id;
    // Arbitrary number of maximum tries just to prevent this optimization from taking too long.
    let max_tries = 5;
    for _ in 0..max_tries {
        match &dfg[array_id] {
            Value::Instruction { instruction, .. } => match &dfg[*instruction] {
                Instruction::ArraySet { array, index, .. } => {
                    let Some(index) = dfg.get_numeric_constant(*index) else {
                        return SimplifyResult::None;
                    };

                    if index == target_index {
                        return SimplifyResult::None;
                    }

                    if *array == array_from_get {
                        return SimplifyResult::SimplifiedTo(original_array_id);
                    }

                    array_id = *array; // recur
                }
                _ => return SimplifyResult::None,
            },
            _ => return SimplifyResult::None,
        }
    }

    SimplifyResult::None
}

#[cfg(test)]
mod tests {
    use crate::ssa::{opt::assert_normalized_ssa_equals, ssa_gen::Ssa};

    #[test]
    fn removes_range_constraints_on_constants() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            range_check Field 0 to 1 bits
            range_check Field 1 to 1 bits
            range_check Field 255 to 8 bits
            range_check Field 256 to 8 bits
            return
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        let expected = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            range_check Field 256 to 8 bits
            return
        }
        ";
        assert_normalized_ssa_equals(ssa, expected);
    }
}
