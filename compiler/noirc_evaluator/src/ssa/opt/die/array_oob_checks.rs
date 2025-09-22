use noirc_errors::call_stack::CallStackId;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        function::Function,
        instruction::{BinaryOp, Instruction},
        types::{NumericType, Type},
        value::ValueId,
    },
    opt::die::Context,
};

impl Context {
    /// Replaces unused ArrayGet/ArraySet instructions with out of bounds checks.
    /// Returns `true` if at least one check was inserted.
    /// Because some ArrayGet might happen in groups (for composite types), if just
    /// some of the instructions in a group are used but not all of them, no check
    /// is inserted, so this method might return `false`.
    pub(super) fn replace_array_instructions_with_out_of_bounds_checks(
        &mut self,
        function: &mut Function,
        block_id: BasicBlockId,
        possible_index_out_of_bounds_indexes: &mut Vec<usize>,
    ) -> bool {
        let mut inserted_check = false;

        // Keep track of the current side effects condition
        let mut side_effects_condition = None;

        // Keep track of the next index we need to handle
        let mut next_out_of_bounds_index = possible_index_out_of_bounds_indexes.pop();

        let instructions = function.dfg[block_id].take_instructions();
        for (index, instruction_id) in instructions.iter().enumerate() {
            let instruction_id = *instruction_id;
            let instruction = &function.dfg[instruction_id];

            if let Instruction::EnableSideEffectsIf { condition } = instruction {
                side_effects_condition = Some(*condition);

                // We still need to keep the EnableSideEffects instruction
                function.dfg[block_id].instructions_mut().push(instruction_id);
                continue;
            };

            // If it's an ArrayGet we'll deal with groups of it in case the array type is a composite type,
            // and adjust `next_out_of_bounds_index` and `possible_index_out_of_bounds_indexes` accordingly
            if let Instruction::ArrayGet { array, .. } = instruction {
                handle_array_get_group(
                    function,
                    array,
                    index,
                    &mut next_out_of_bounds_index,
                    possible_index_out_of_bounds_indexes,
                );
            }

            let Some(out_of_bounds_index) = next_out_of_bounds_index else {
                // No more out of bounds instructions to insert, just push the current instruction
                function.dfg[block_id].instructions_mut().push(instruction_id);
                continue;
            };

            if index != out_of_bounds_index {
                // This instruction is not out of bounds: let's just push it
                function.dfg[block_id].instructions_mut().push(instruction_id);
                continue;
            }

            // This is an instruction that might be out of bounds: let's add a constrain.
            let (array, index) = match instruction {
                Instruction::ArrayGet { array, index, .. }
                | Instruction::ArraySet { array, index, .. } => (array, index),
                _ => panic!("Expected an ArrayGet or ArraySet instruction here"),
            };

            let call_stack = function.dfg.get_instruction_call_stack_id(instruction_id);

            let (lhs, rhs) = if function.dfg.get_numeric_constant(*index).is_some() {
                // If we are here it means the index is known but out of bounds. That's always an error!
                let false_const = function.dfg.make_constant(false.into(), NumericType::bool());
                let true_const = function.dfg.make_constant(true.into(), NumericType::bool());
                (false_const, true_const)
            } else {
                let array_typ = function.dfg.type_of_value(*array);
                let element_size = array_typ.element_size() as u32;
                let len = match array_typ {
                    Type::Array(_, len) => len,
                    _ => panic!("Expected an array"),
                };
                // `index` will be relative to the flattened array length, so we need to take that into account
                let array_length = element_size * len;
                // let array_length = function.dfg.type_of_value(*array).flattened_size();

                // If we are here it means the index is dynamic, so let's add a check that it's less than length
                let length_type = NumericType::length_type();
                let index = function.dfg.insert_instruction_and_results(
                    Instruction::Cast(*index, length_type),
                    block_id,
                    None,
                    call_stack,
                );
                let index = index.first();
                let array_length =
                    function.dfg.make_constant(u128::from(array_length).into(), length_type);

                let is_index_out_of_bounds = function.dfg.insert_instruction_and_results(
                    Instruction::binary(BinaryOp::Lt, index, array_length),
                    block_id,
                    None,
                    call_stack,
                );
                let is_index_out_of_bounds = is_index_out_of_bounds.first();
                let true_const = function.dfg.make_constant(true.into(), NumericType::bool());
                (is_index_out_of_bounds, true_const)
            };

            let (lhs, rhs) = apply_side_effects(
                side_effects_condition,
                lhs,
                rhs,
                function,
                block_id,
                call_stack,
            );

            let message = Some("Index out of bounds".to_owned().into());
            function.dfg.insert_instruction_and_results(
                Instruction::Constrain(lhs, rhs, message),
                block_id,
                None,
                call_stack,
            );
            inserted_check = true;

            next_out_of_bounds_index = possible_index_out_of_bounds_indexes.pop();
        }

        inserted_check
    }
}

/// Array get/set has explicit out of bounds (OOB) checks laid down in the Brillig runtime.
/// These checks are not laid down in the ACIR runtime as that runtime maps the SSA
/// to a memory model where OOB accesses will be prevented. Essentially all array ops
/// in ACIR will have a side effect where they check for the index being OOB.
/// However, in order to maintain parity between the Brillig and ACIR runtimes,
/// if we have an unused array operation we need insert an OOB check so that the
/// side effects ordering remains correct.
pub(super) fn should_insert_oob_check(function: &Function, instruction: &Instruction) -> bool {
    if !function.runtime().is_acir() {
        return false;
    }

    use Instruction::*;
    match instruction {
        ArrayGet { array, index } | ArraySet { array, index, .. } => {
            // We only care about arrays here as slices are expected to have explicit checks laid down in the initial SSA.
            function.dfg.try_get_array_length(*array).is_some()
                && !function.dfg.is_safe_index(*index, *array)
        }
        _ => false,
    }
}

pub(super) fn handle_array_get_group(
    function: &Function,
    array: &ValueId,
    index: usize,
    next_out_of_bounds_index: &mut Option<usize>,
    possible_index_out_of_bounds_indexes: &mut Vec<usize>,
) {
    if function.dfg.try_get_array_length(*array).is_none() {
        // Nothing to do for slices
        return;
    };

    let element_size = function.dfg.type_of_value(*array).element_size();
    if element_size <= 1 {
        // Not a composite type
        return;
    };

    // It's a composite type.
    // When doing ArrayGet on a composite type, this **always** results in instructions like these
    // (assuming element_size == 3):
    //
    // 1.    v27 = array_get v1, index v26
    // 2.    v28 = add v26, u32 1
    // 3.    v29 = array_get v1, index v28
    // 4.    v30 = add v26, u32 2
    // 5.    v31 = array_get v1, index v30
    //
    // That means that after this instructions, (element_size - 1) instructions will be
    // part of this composite array get, and they'll be two instructions apart.
    //
    // Now three things can happen:
    // a) none of the array_get instructions are unused: in this case they won't be in
    //    `possible_index_out_of_bounds_indexes` and they won't be removed, nothing to do here
    // b) all of the array_get instructions are unused: in this case we can replace **all**
    //    of them with just one constrain: no need to do one per array_get
    // c) some of the array_get instructions are unused, but not all: in this case
    //    we don't need to insert any constrain, because on a later stage array bound checks
    //    will be performed anyway. We'll let DIE remove the unused ones, without replacing
    //    them with bounds checks, and leave the used ones.
    //
    // To check in which scenario we are we can get from `possible_index_out_of_bounds_indexes`
    // (starting from `next_out_of_bounds_index`) while we are in the group ranges
    // (1..=5 in the example above)

    let Some(out_of_bounds_index) = *next_out_of_bounds_index else {
        // No next unused instruction, so this is case a) and nothing needs to be done here
        return;
    };

    if index != out_of_bounds_index {
        // The next index is not the one for the current instructions,
        // so we are in case a), and nothing needs to be done here
        return;
    }

    // What's the last instruction that's part of the group? (5 in the example above)
    let last_instruction_index = index + 2 * (element_size - 1);
    // How many unused instructions are in this group?
    let mut unused_count = 1;
    loop {
        *next_out_of_bounds_index = possible_index_out_of_bounds_indexes.pop();
        if let Some(out_of_bounds_index) = *next_out_of_bounds_index {
            if out_of_bounds_index <= last_instruction_index {
                unused_count += 1;
                if unused_count == element_size {
                    // We are in case b): we need to insert just one constrain.
                    // Since we popped all of the group indexes, and given that we
                    // are analyzing the first instruction in the group, we can
                    // set `next_out_of_bounds_index` to the current index:
                    // then a check will be inserted, and no other check will be
                    // inserted for the rest of the group.
                    *next_out_of_bounds_index = Some(index);
                    break;
                } else {
                    continue;
                }
            }
        }

        // We are in case c): some of the instructions are unused.
        // We don't need to insert any checks, and given that we already popped
        // all of the indexes in the group, there's nothing else to do here.
        break;
    }
}

// Given `lhs` and `rhs` values, if there's a side effects condition this will
// return (`lhs * condition`, `rhs * condition`), otherwise just (`lhs`, `rhs`)
fn apply_side_effects(
    side_effects_condition: Option<ValueId>,
    lhs: ValueId,
    rhs: ValueId,
    function: &mut Function,
    block_id: BasicBlockId,
    call_stack: CallStackId,
) -> (ValueId, ValueId) {
    // See if there's an active "enable side effects" condition
    let Some(condition) = side_effects_condition else {
        return (lhs, rhs);
    };

    let dfg = &mut function.dfg;

    // Condition needs to be cast to argument type in order to multiply them together.
    // In our case, lhs is always a boolean.
    let cast = Instruction::Cast(condition, NumericType::bool());
    let casted_condition = dfg.insert_instruction_and_results(cast, block_id, None, call_stack);
    let casted_condition = casted_condition.first();

    // Unchecked mul because the side effects var is always 0 or 1
    let lhs = dfg.insert_instruction_and_results(
        Instruction::binary(BinaryOp::Mul { unchecked: true }, lhs, casted_condition),
        block_id,
        None,
        call_stack,
    );
    let lhs = lhs.first();

    // Unchecked mul because the side effects var is always 0 or 1
    let rhs = dfg.insert_instruction_and_results(
        Instruction::binary(BinaryOp::Mul { unchecked: true }, rhs, casted_condition),
        block_id,
        None,
        call_stack,
    );
    let rhs = rhs.first();

    (lhs, rhs)
}
