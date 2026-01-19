use noirc_errors::call_stack::CallStackId;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        function::Function,
        instruction::{Binary, BinaryOp, Instruction, InstructionId},
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
                    &instructions,
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
                let element_size = array_typ.element_size();
                let len = match array_typ {
                    Type::Array(_, len) => len,
                    _ => panic!("Expected an array"),
                };
                // `index` will be relative to the flattened array length, so we need to take that into account
                let array_length = element_size * len;

                // If we are here it means the index is dynamic, so let's add a check that it's less than length.

                // Normally the indexes are expected to be u32, however if the array element is a composite type,
                // the value could have overflown due to the unchecked multiplication with the element size.
                // In ACIR, we rely on the array operation itself to fail the circuit if it encounters an overflown value,
                // however we are just removing the array operation and replacing it with a LessThan, which in ACIR gen
                // lays down an RangeCheck that would fail if the value doesn't fit 32 bits. As a workaround,
                // instead of finding the index instruction and changing into a checked multiplication,
                // we cast to a higher bitsize, which we expect should fit any overflown index type.
                let length_type = NumericType::unsigned(64);

                let index = function.dfg.insert_instruction_and_results(
                    Instruction::Cast(*index, length_type),
                    block_id,
                    None,
                    call_stack,
                );
                let index = index.first();
                let array_length = function.dfg.make_constant(array_length.0.into(), length_type);

                let is_index_in_bounds = function.dfg.insert_instruction_and_results(
                    Instruction::binary(BinaryOp::Lt, index, array_length),
                    block_id,
                    None,
                    call_stack,
                );
                let is_index_in_bounds = is_index_in_bounds.first();
                let true_const = function.dfg.make_constant(true.into(), NumericType::bool());
                (is_index_in_bounds, true_const)
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
            // We only care about arrays here as vectors are expected to have explicit checks laid down in the initial SSA.
            function.dfg.try_get_array_length(*array).is_some()
                && !function.dfg.is_safe_index(*index, *array)
        }
        _ => false,
    }
}

/// Handle the case when an `ArrayGet` is potentially out-of-bounds and the array contains composite types
/// by figuring out whether all `ArrayGet` of different parts of the complex item are unused, and if so
/// then insert a single constraint to replace all of them.
///
/// Consumes all items from `possible_index_out_of_bounds_indexes` that belong to the current group and
/// sets `next_out_of_bounds_index` to the *current* index, expecting that `replace_array_instructions_with_out_of_bounds_checks`
/// will see that as a signal that the current index is out of bounds and it should insert a constraint.
/// Then, the next a `ArrayGet`s in the group will be re-inserted, but they won't be treated as potentially
/// OOB any more, and shall be removed in the next DIE pass as simply unused.
fn handle_array_get_group(
    function: &Function,
    // The array from which we are getting an item.
    array: &ValueId,
    // Index of the current instruction. If it's not the same as `Some(next_out_of_bounds_index)`
    // then this instruction was not unsafe.
    index: usize,
    // The last index popped from `possible_index_out_of_bounds_indexes`.
    next_out_of_bounds_index: &mut Option<usize>,
    // Remaining out of bounds indexes, all of which are unused.
    possible_index_out_of_bounds_indexes: &mut Vec<usize>,
    // All the instructions in this block.
    instructions: &[InstructionId],
) {
    if function.dfg.try_get_array_length(*array).is_none() {
        // Nothing to do for vectors
        return;
    };

    let element_size = function.dfg.type_of_value(*array).element_size().to_usize();
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
    // However, that is only true for the initial SSA. After we run DIE, it might remove
    // some of the instructions that were unused, leaving the ones which had uses, destroying
    // the group, so in general we cannot assume to see all element_size instruction to be present.
    //
    // Assuming we have identified a group, three things can happen:
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

    // Initially we would expect the last index of the group (5 in the example above)
    // to be `index + 2 * (element_size - 1)`, however, we can't expect this to hold
    // after previous DIE passes have partially removed the group.
    let last_possible_index = index + 2 * (element_size - 1);
    // Instead we need to check how many `ArrayGet` and `Add` we have following this
    // instruction that read the same array, and how many of these instructions are unused.
    let max_index = last_possible_index.min(instructions.len() - 1);

    // How many unused instructions are in this group? We know the current instruction is unused.
    let mut unused_count = 1;
    let mut group_count = 1;

    for (i, next_id) in instructions.iter().enumerate().take(max_index + 1).skip(index + 1) {
        let next_instruction = &function.dfg[*next_id];
        match next_instruction {
            // Skip `Add`
            Instruction::Binary(Binary { operator: BinaryOp::Add { .. }, .. }) => {
                continue;
            }
            Instruction::ArrayGet { array: next_array, index: next_index }
                if next_array == array =>
            {
                // Still reading the same array.
                // There is a chance that *this* instruction is safe, which means the one before it
                // needs to be replaced with a constraint, even if this does not.
                if function.dfg.is_safe_index(*next_index, *next_array) {
                    break;
                }
                // This instruction is also OOB, so it belongs to the same group.
                group_count += 1;
                // Check if this result is also unused.
                *next_out_of_bounds_index = possible_index_out_of_bounds_indexes.pop();
                let Some(out_of_bounds_index) = *next_out_of_bounds_index else {
                    // This ArrayGet is not recorded as a potential OOB; we know it's OOB, so this means it's not unused.
                    // That means we can let the built-in OOB check take care of it.
                    break;
                };
                if out_of_bounds_index == i {
                    unused_count += 1;
                    continue;
                } else {
                    // The next OOB index is for some other array, not this one; the last array get
                    // reading this array is not OOB or not unused.
                    break;
                }
            }
            _ => {
                // Some other instruction that doesn't belong to the group.
                break;
            }
        }
    }

    if unused_count == group_count {
        // We are in case b): we need to insert just one constrain.
        // Since we popped all of the group indexes, and given that we
        // are analyzing the first instruction in the group, we can
        // set `next_out_of_bounds_index` to the current index:
        // then a check will be inserted, and no other check will be
        // inserted for the rest of the group.
        *next_out_of_bounds_index = Some(index);
    } else {
        // We are in case c): some of the instructions are unused.
        // We don't need to insert any checks, and given that we already popped
        // all of the indexes in the group, there's nothing else to do here.
    }
}

/// Given `lhs` and `rhs` values, if there's a side effects condition this will
/// return (`lhs * condition`, `rhs * condition`), otherwise just (`lhs`, `rhs`)
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
