use crate::ssa::ir::{
    basic_block::BasicBlockId,
    dfg::simplify::value_merger::ValueMerger,
    instruction::{
        Binary, BinaryOp, ConstrainError, Instruction,
        binary::{truncate, truncate_field},
    },
    types::{NumericType, Type},
    value::{Value, ValueId},
};
use acvm::{
    AcirField as _, FieldElement,
    acir::brillig::lengths::{ElementTypesLength, SemanticLength},
};
use binary::simplify_binary;
use call::simplify_call;
use cast::simplify_cast;
use constrain::decompose_constrain;
use noirc_errors::call_stack::CallStackId;

use super::DataFlowGraph;

mod binary;
mod call;
mod cast;
mod constrain;
pub(crate) mod value_merger;

pub(crate) use call::constant_to_radix;

/// Contains the result to Instruction::simplify, specifying how the instruction
/// should be simplified.
#[derive(Debug)]
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
        Instruction::Binary(binary) => simplify_binary(binary, dfg, block, call_stack),
        Instruction::Cast(value, typ) => simplify_cast(*value, *typ, dfg),
        Instruction::Not(value) => {
            match &dfg[*value] {
                // Limit optimizing ! on constants to only booleans. If we tried it on fields,
                // there is no Not on FieldElement, so we'd need to convert between u128. This
                // would be incorrect however since the extra bits on the field would not be flipped.
                Value::NumericConstant { constant, typ } if typ.is_unsigned() => {
                    // As we're casting to a `u128`, we need to clear out any upper bits that the NOT fills.
                    let bit_size = typ.bit_size::<FieldElement>();
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
                return try_optimize_array_get_from_previous_set(dfg, *array, index);
            }

            let array_or_vector_type = dfg.type_of_value(*array);
            if matches!(array_or_vector_type, Type::Array(_, SemanticLength(1)))
                && array_or_vector_type.element_size() == ElementTypesLength(1)
            {
                // If the array is of length 1 then we know the only value which can be potentially read out of it.
                // We can then simply assert that the index is equal to zero and return the array's contained value.
                optimize_length_one_array_read(dfg, block, call_stack, *array, *index)
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
            if bit_size >= max_bit_size {
                return SimplifiedTo(*value);
            }
            if let Some((numeric_constant, typ)) = dfg.get_numeric_constant_with_type(*value) {
                let truncated_field = truncate_field(numeric_constant, *bit_size);
                SimplifiedTo(dfg.make_constant(truncated_field, typ))
            } else if let Value::Instruction { instruction, .. } = &dfg[*value] {
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
                        let numerator_smaller_than_denominator = max_numerator_bits
                            .checked_sub(divisor_bits)
                            .is_some_and(|max_quotient_bits| max_quotient_bits < *bit_size);
                        if numerator_smaller_than_denominator { SimplifiedTo(*value) } else { None }
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
        Instruction::Allocate => None,
        Instruction::Load { .. } => None,
        Instruction::Store { .. } => None,
        Instruction::IncrementRc { .. } => None,
        Instruction::DecrementRc { .. } => None,
        Instruction::RangeCheck { value, max_bit_size, assert_message } => {
            let max_potential_bits = dfg.get_value_max_num_bits(*value);
            if max_potential_bits <= *max_bit_size {
                Remove
            } else if *max_bit_size == 0 {
                let typ = dfg.type_of_value(*value).unwrap_numeric();
                let zero = dfg.make_constant(FieldElement::zero(), typ);
                SimplifiedToInstruction(Instruction::Constrain(
                    *value,
                    zero,
                    assert_message.as_ref().map(|msg| ConstrainError::from(msg.clone())),
                ))
            } else if let Some(c) = dfg.get_numeric_constant(*value) {
                if c.num_bits() > *max_bit_size {
                    let zero = dfg.make_constant(FieldElement::zero(), NumericType::bool());
                    let one = dfg.make_constant(FieldElement::one(), NumericType::bool());
                    SimplifiedToInstruction(Instruction::Constrain(
                        zero,
                        one,
                        assert_message.as_ref().map(|msg| ConstrainError::from(msg.clone())),
                    ))
                } else {
                    Remove
                }
            } else {
                None
            }
        }
        Instruction::IfElse { then_condition, then_value, else_condition, else_value } => {
            let then_condition = *then_condition;
            let else_condition = *else_condition;
            let typ = dfg.type_of_value(*then_value);

            if let Some(constant) = dfg.get_numeric_constant(then_condition) {
                if constant.is_one() {
                    return SimplifiedTo(*then_value);
                } else if constant.is_zero() {
                    return SimplifiedTo(*else_value);
                }
            }

            let then_value = *then_value;
            let else_value = *else_value;
            if then_value == else_value {
                return SimplifiedTo(then_value);
            }

            if let Some(Instruction::IfElse {
                then_condition: inner_then_condition,
                then_value: inner_then_value,
                ..
            }) = dfg.get_local_or_global_instruction(then_value)
            {
                if then_condition == *inner_then_condition {
                    let instruction = Instruction::IfElse {
                        then_condition,
                        then_value: *inner_then_value,
                        else_condition,
                        else_value,
                    };
                    return SimplifiedToInstruction(instruction);
                }
                // TODO: We could check to see if `then_condition == inner_else_condition`
                // but we run into issues with duplicate NOT instructions having distinct ValueIds.
            }

            if let Some(Instruction::IfElse {
                then_condition: inner_then_condition,
                else_value: inner_else_value,
                ..
            }) = dfg.get_local_or_global_instruction(else_value)
            {
                if then_condition == *inner_then_condition {
                    let instruction = Instruction::IfElse {
                        then_condition,
                        then_value,
                        else_condition,
                        else_value: *inner_else_value,
                    };
                    return SimplifiedToInstruction(instruction);
                }
                // TODO: We could check to see if `then_condition == inner_else_condition`
                // but we run into issues with duplicate NOT instructions having distinct ValueIds.
            }

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

/// Given an array access on a length 1 array such as:
/// ```ssa
/// v2 = make_array [v0] : [Field; 1]
/// v3 = array_get v2, index v1 -> Field
/// ```
///
/// We want to replace the array read with the only valid value which can be read from the array
/// while ensuring that if there is an attempt to read past the end of the array then the program fails.
///
/// We then inject an explicit assertion that the index variable has the value zero while replacing the value
/// being used in the `array_get` instruction with a constant value of zero. This then results in the SSA:
///
/// ```ssa
/// v2 = make_array [v0] : [Field; 1]
/// constrain v1 == u32 0, "Index out of bounds"
/// v4 = array_get v2, index u32 0 -> Field
/// ```
/// We then attempt to resolve the array read immediately.
///
/// Note that this does not work if the array has length 1, but contains a complex type such as tuple,
/// which consists of multiple elements. If that is the case than the `index` will most likely not be
/// a constant, but a base plus an offset, and if the array contains repeated elements of the same type
/// for example, we wouldn't be able to come up with a constant offset even if we knew the return type.
fn optimize_length_one_array_read(
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    call_stack: CallStackId,
    array: ValueId,
    index: ValueId,
) -> SimplifyResult {
    let zero = dfg.make_constant(FieldElement::zero(), NumericType::length_type());
    let index_constraint = Instruction::Constrain(
        index,
        zero,
        Some(ConstrainError::from("Index out of bounds".to_string())),
    );
    dfg.insert_instruction_and_results(index_constraint, block, None, call_stack);

    let result = try_optimize_array_get_from_previous_set(dfg, array, FieldElement::zero());
    if let SimplifyResult::None = result {
        SimplifyResult::SimplifiedToInstruction(Instruction::ArrayGet { array, index: zero })
    } else {
        result
    }
}

/// Given a chain of operations like:
/// v1 = array_set [10, 11, 12], index 1, value: 5
/// v2 = array_set v1, index 2, value: 6
/// v3 = array_set v2, index 2, value: 7
/// v4 = array_get v3, index 1
///
/// We want to optimize `v4` to `11`. To do this we need to follow the array value
/// through several array sets. For each array set:
/// - If the index is non-constant we fail the optimization since any index may be changed
/// - If the index is constant and is our target index, we conservatively fail the optimization
///   in case the array_set is disabled from a previous `enable_side_effects_if` and the array get
///   was not.
/// - Otherwise, we check the array value of the array set.
///   - If the array value is constant, we use that array.
///   - If the array value is from a previous array-set, we recur.
///   - If the array value is from an array parameter, we use that array.
///
/// That is, we have multiple `array_set` instructions setting various constant indexes
/// of the same array, returning a modified version. We want to go backwards until we
/// find the last `array_set` for the index we are interested in, and return the value set.
fn try_optimize_array_get_from_previous_set(
    dfg: &mut DataFlowGraph,
    mut array_id: ValueId,
    target_index: FieldElement,
) -> SimplifyResult {
    // The target index must be less than the maximum array length
    let Some(target_index_u32) = target_index.try_to_u32() else {
        return SimplifyResult::None;
    };

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
                        continue;
                    }
                }
                Instruction::MakeArray { elements: array, typ: _ } => {
                    let index = target_index_u32 as usize;
                    if index < array.len() {
                        return SimplifyResult::SimplifiedTo(array[index]);
                    }
                }
                _ => (),
            }
        } else if let Value::Param { typ: Type::Array(_, length), .. } = &dfg[array_id] {
            if target_index_u32 < length.0 {
                let index = dfg.make_constant(target_index, NumericType::length_type());
                return SimplifyResult::SimplifiedToInstruction(Instruction::ArrayGet {
                    array: array_id,
                    index,
                });
            }
        }

        break;
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
    let array_from_get = match dfg.get_local_or_global_instruction(target_value) {
        Some(Instruction::ArrayGet { array, index }) => {
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
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_normalized_ssa_equals, ssa_gen::Ssa},
    };

    #[test]
    fn removes_range_constraints_on_constants() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: u8):
            range_check Field 0 to 1 bits
            range_check Field 1 to 1 bits
            range_check Field 2 to 1 bits, "2 > 1"
            range_check Field 255 to 8 bits
            range_check Field 256 to 8 bits, "256 > 255"
            range_check v0 to 8 bits
            range_check v1 to 8 bits
            return
        }
        "#;
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: u8):
            constrain u1 0 == u1 1, "2 > 1"
            constrain u1 0 == u1 1, "256 > 255"
            range_check v0 to 8 bits
            return
        }
        "#);
    }

    #[test]
    fn simplifies_or_when_one_side_is_all_1s() {
        let test_cases = vec![
            ("u128", u128::MAX.to_string()),
            ("u64", u64::MAX.to_string()),
            ("u32", u32::MAX.to_string()),
            ("u16", u16::MAX.to_string()),
            ("u8", u8::MAX.to_string()),
        ];
        const SRC_TEMPLATE: &str = "
        acir(inline) pure fn main f0 {
          b0(v1: {typ}):
            v2 = or {typ} {max}, v1
            return v2
        }
        ";

        const EXPECTED_TEMPLATE: &str = "
        acir(inline) pure fn main f0 {
          b0(v1: {typ}):
            return {typ} {max}
        }
        ";
        for (typ, max) in test_cases {
            let src = SRC_TEMPLATE.replace("{typ}", typ).replace("{max}", &max);
            let expected = EXPECTED_TEMPLATE.replace("{typ}", typ).replace("{max}", &max);
            let ssa: Ssa = Ssa::from_str_simplifying(&src).unwrap();
            assert_normalized_ssa_equals(ssa, &expected);
        }
    }

    #[test]
    fn simplifies_noop_bitwise_and_truncation() {
        let test_cases = vec![
            ("u128", u128::MAX.to_string()),
            ("u64", u64::MAX.to_string()),
            ("u32", u32::MAX.to_string()),
            ("u16", u16::MAX.to_string()),
            ("u8", u8::MAX.to_string()),
        ];
        const SRC_TEMPLATE: &str = "
        acir(inline) pure fn main f0 {
          b0(v1: {typ}):
            v2 = and {typ} {max}, v1
            return v2
        }
        ";

        const EXPECTED_TEMPLATE: &str = "
        acir(inline) pure fn main f0 {
          b0(v1: {typ}):
            return v1
        }
        ";
        for (typ, max) in test_cases {
            let src = SRC_TEMPLATE.replace("{typ}", typ).replace("{max}", &max);
            let expected = EXPECTED_TEMPLATE.replace("{typ}", typ);
            let ssa: Ssa = Ssa::from_str_simplifying(&src).unwrap();
            assert_normalized_ssa_equals(ssa, &expected);
        }
    }

    #[test]
    fn truncate_to_bit_size_bigger_than_value_max_bit_size() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u8):
            v1 = truncate v0 to 16 bits, max_bit_size: 8
            v2 = cast v1 as u16
            return v2
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u8):
            v1 = cast v0 as u16
            return v1
        }
        ");
    }

    #[test]
    fn replaces_length_one_array_get_with_bounds_check() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: u32):
            v2 = make_array [v0] : [Field; 1]
            v3 = array_get v2, index v1 -> Field
            return v3
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: u32):
            v2 = make_array [v0] : [Field; 1]
            constrain v1 == u32 0, "Index out of bounds"
            return v0
        }
        "#);
    }

    #[test]
    fn does_not_use_flattened_size_for_length_one_array_check() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: u32):
            v2 = make_array [v0, v0] : [Field; 2]
            v3 = make_array [v2] : [[Field; 2]; 1]
            v4 = make_array [] : [Field; 0]
            v5 = make_array [v4, v0] : [([Field; 0], Field); 1]
            v6 = array_get v3, index v1 -> [Field; 2]
            v7 = add v1, u32 1
            v8 = array_get v5, index v7 -> Field
            return v6, v8
        }
        ";
        // The flattened size of v3 is 2, but it has 1 element -> it can be optimized.
        // The flattened size of v5 is 1, but it has 2 elements -> it cannot be optimized.

        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: u32):
            v2 = make_array [v0, v0] : [Field; 2]
            v3 = make_array [v2] : [[Field; 2]; 1]
            v4 = make_array [] : [Field; 0]
            v5 = make_array [v4, v0] : [([Field; 0], Field); 1]
            constrain v1 == u32 0, "Index out of bounds"
            v8 = add v1, u32 1
            v9 = array_get v5, index v8 -> Field
            return v2, v9
        }
        "#);
    }

    #[test]
    fn does_not_crash_on_truncated_division_with_large_denominators() {
        // There can be invalid division instructions which have extremely large denominators
        // so we want to make sure that we handle this case when optimizing truncations.

        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = div i8 94, i8 19807040628566084398385987584
            v1 = truncate v0 to 8 bits, max_bit_size: 9
            return v1
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn simplifies_array_get_from_previous_array_set_with_make_array() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [Field 2, Field 3] : [Field; 2]
            v1 = array_set mut v0, index u32 0, value Field 4
            v2 = array_get v1, index u32 0 -> Field
            v3 = array_get v1, index u32 1 -> Field
            return v2, v3
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v2 = make_array [Field 2, Field 3] : [Field; 2]
            v4 = make_array [Field 4, Field 3] : [Field; 2]
            return Field 4, Field 3
        }
        ");
    }

    #[test]
    fn simplifies_array_get_from_previous_array_set_with_array_param_in_bounds() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [Field; 2]):
            v1 = array_set mut v0, index u32 0, value Field 4
            v2 = array_get v1, index u32 0 -> Field
            v3 = array_get v1, index u32 1 -> Field
            return v2, v3
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [Field; 2]):
            v3 = array_set mut v0, index u32 0, value Field 4
            v5 = array_get v0, index u32 1 -> Field
            return Field 4, v5
        }
        ");
    }

    #[test]
    fn does_not_simplify_array_get_from_previous_array_set_with_array_param_out_of_bounds() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [Field; 2]):
            v3 = array_set mut v0, index u32 0, value Field 4
            v5 = array_get v3, index u32 2 -> Field
            return v5
        }
        ";
        let ssa = Ssa::from_str_simplifying(src).unwrap();
        assert_normalized_ssa_equals(ssa, src);
    }
}
