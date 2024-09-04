//! Dead Instruction Elimination (DIE) pass: Removes any instruction without side-effects for
//! which the results are unused.
use std::collections::HashSet;

use im::Vector;
use noirc_errors::Location;

use crate::ssa::{
    ir::{
        basic_block::{BasicBlock, BasicBlockId},
        dfg::DataFlowGraph,
        function::Function,
        instruction::{BinaryOp, Instruction, InstructionId, Intrinsic},
        post_order::PostOrder,
        types::Type,
        value::{Value, ValueId},
    },
    ssa_gen::{Ssa, SSA_WORD_SIZE},
};

impl Ssa {
    /// Performs Dead Instruction Elimination (DIE) to remove any instructions with
    /// unused results.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn dead_instruction_elimination(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            dead_instruction_elimination(function, true);
        }
        self
    }
}

/// Removes any unused instructions in the reachable blocks of the given function.
///
/// The blocks of the function are iterated in post order, such that any blocks containing
/// instructions that reference results from an instruction in another block are evaluated first.
/// If we did not iterate blocks in this order we could not safely say whether or not the results
/// of its instructions are needed elsewhere.
fn dead_instruction_elimination(function: &mut Function, insert_out_of_bounds_checks: bool) {
    let mut context = Context::default();
    for call_data in &function.dfg.data_bus.call_data {
        context.mark_used_instruction_results(&function.dfg, call_data.array_id);
    }

    let mut inserted_out_of_bounds_checks = false;

    let blocks = PostOrder::with_function(function);
    for block in blocks.as_slice() {
        inserted_out_of_bounds_checks |= context.remove_unused_instructions_in_block(
            function,
            *block,
            insert_out_of_bounds_checks,
        );
    }

    // If we inserted out of bounds check, let's run the pass again with those new
    // instructions (we don't want to remove those checks, or instructions that are
    // dependencies of those checks)
    if inserted_out_of_bounds_checks {
        dead_instruction_elimination(function, false);
        return;
    }

    context.remove_rc_instructions(&mut function.dfg);
}

/// Per function context for tracking unused values and which instructions to remove.
#[derive(Default)]
struct Context {
    used_values: HashSet<ValueId>,
    instructions_to_remove: HashSet<InstructionId>,

    /// IncrementRc & DecrementRc instructions must be revisited after the main DIE pass since
    /// they technically contain side-effects but we still want to remove them if their
    /// `value` parameter is not used elsewhere.
    rc_instructions: Vec<(InstructionId, BasicBlockId)>,
}

impl Context {
    /// Steps backwards through the instruction of the given block, amassing a set of used values
    /// as it goes, and at the same time marking instructions for removal if they haven't appeared
    /// in the set thus far.
    ///
    /// It is not only safe to mark instructions for removal as we go because no instruction
    /// result value can be referenced before the occurrence of the instruction that produced it,
    /// and we are iterating backwards. It is also important to identify instructions that can be
    /// removed as we go, such that we know not to include its referenced values in the used
    /// values set. This allows DIE to identify whole chains of unused instructions. (If the
    /// values referenced by an unused instruction were considered to be used, only the head of
    /// such chains would be removed.)
    ///
    /// If `insert_out_of_bounds_checks` is true and there are unused ArrayGet/ArraySet that
    /// might be out of bounds, this method will insert out of bounds checks instead of
    /// removing unused instructions and return `true`. The idea then is to later call this
    /// function again with `insert_out_of_bounds_checks` set to false to effectively remove
    /// unused instructions but leave the out of bounds checks.
    fn remove_unused_instructions_in_block(
        &mut self,
        function: &mut Function,
        block_id: BasicBlockId,
        insert_out_of_bounds_checks: bool,
    ) -> bool {
        let block = &function.dfg[block_id];
        self.mark_terminator_values_as_used(function, block);

        let instructions_len = block.instructions().len();

        // Indexes of instructions that might be out of bounds.
        // We'll remove those, but before that we'll insert bounds checks for them.
        let mut possible_index_out_of_bounds_indexes = Vec::new();

        for (instruction_index, instruction_id) in block.instructions().iter().rev().enumerate() {
            let instruction = &function.dfg[*instruction_id];

            if self.is_unused(*instruction_id, function) {
                self.instructions_to_remove.insert(*instruction_id);

                if insert_out_of_bounds_checks
                    && instruction_might_result_in_out_of_bounds(function, instruction)
                {
                    possible_index_out_of_bounds_indexes
                        .push(instructions_len - instruction_index - 1);
                }
            } else {
                use Instruction::*;
                if matches!(instruction, IncrementRc { .. } | DecrementRc { .. }) {
                    self.rc_instructions.push((*instruction_id, block_id));
                } else {
                    instruction.for_each_value(|value| {
                        self.mark_used_instruction_results(&function.dfg, value);
                    });
                }
            }
        }

        // If there are some instructions that might trigger an out of bounds error,
        // first add constrain checks. Then run the DIE pass again, which will remove those
        // but leave the constrains (any any value needed by those constrains)
        if !possible_index_out_of_bounds_indexes.is_empty() {
            let inserted_check = self.replace_array_instructions_with_out_of_bounds_checks(
                function,
                block_id,
                &mut possible_index_out_of_bounds_indexes,
            );
            // There's a slight chance we didn't insert any checks, so we could proceed with DIE.
            if inserted_check {
                return true;
            }
        }

        function.dfg[block_id]
            .instructions_mut()
            .retain(|instruction| !self.instructions_to_remove.contains(instruction));

        false
    }

    /// Returns true if an instruction can be removed.
    ///
    /// An instruction can be removed as long as it has no side-effects, and none of its result
    /// values have been referenced.
    fn is_unused(&self, instruction_id: InstructionId, function: &Function) -> bool {
        let instruction = &function.dfg[instruction_id];

        if instruction.can_eliminate_if_unused(&function.dfg) {
            let results = function.dfg.instruction_results(instruction_id);
            results.iter().all(|result| !self.used_values.contains(result))
        } else if let Instruction::Call { func, arguments } = instruction {
            // TODO: make this more general for instructions which don't have results but have side effects "sometimes" like `Intrinsic::AsWitness`
            let as_witness_id = function.dfg.get_intrinsic(Intrinsic::AsWitness);
            as_witness_id == Some(func) && !self.used_values.contains(&arguments[0])
        } else {
            // If the instruction has side effects we should never remove it.
            false
        }
    }

    /// Adds values referenced by the terminator to the set of used values.
    fn mark_terminator_values_as_used(&mut self, function: &Function, block: &BasicBlock) {
        block.unwrap_terminator().for_each_value(|value| {
            self.mark_used_instruction_results(&function.dfg, value);
        });
    }

    /// Inspects a value recursively (as it could be an array) and marks all comprised instruction
    /// results as used.
    fn mark_used_instruction_results(&mut self, dfg: &DataFlowGraph, value_id: ValueId) {
        let value_id = dfg.resolve(value_id);
        match &dfg[value_id] {
            Value::Instruction { .. } => {
                self.used_values.insert(value_id);
            }
            Value::Array { array, .. } => {
                for elem in array {
                    self.mark_used_instruction_results(dfg, *elem);
                }
            }
            Value::Param { .. } => {
                self.used_values.insert(value_id);
            }
            _ => {
                // Does not comprise of any instruction results
            }
        }
    }

    fn remove_rc_instructions(self, dfg: &mut DataFlowGraph) {
        for (rc, block) in self.rc_instructions {
            let value = match &dfg[rc] {
                Instruction::IncrementRc { value } => *value,
                Instruction::DecrementRc { value } => *value,
                other => {
                    unreachable!("Expected IncrementRc or DecrementRc instruction, found {other:?}")
                }
            };

            // This could be more efficient if we have to remove multiple instructions in a single block
            if !self.used_values.contains(&value) {
                dfg[block].instructions_mut().retain(|instruction| *instruction != rc);
            }
        }
    }

    /// Replaces unused ArrayGet/ArraySet instructions with out of bounds checks.
    /// Returns `true` if at least one check was inserted.
    /// Because some ArrayGet might happen in groups (for composite types), if just
    /// some of the instructions in a group are used but not all of them, no check
    /// is inserted, so this method might return `false`.
    fn replace_array_instructions_with_out_of_bounds_checks(
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
                Instruction::ArrayGet { array, index }
                | Instruction::ArraySet { array, index, .. } => (array, index),
                _ => panic!("Expected an ArrayGet or ArraySet instruction here"),
            };

            let call_stack = function.dfg.get_call_stack(instruction_id);

            let (lhs, rhs) = if function.dfg.get_numeric_constant(*index).is_some() {
                // If we are here it means the index is known but out of bounds. That's always an error!
                let false_const = function.dfg.make_constant(false.into(), Type::bool());
                let true_const = function.dfg.make_constant(true.into(), Type::bool());
                (false_const, true_const)
            } else {
                // `index` will be relative to the flattened array length, so we need to take that into account
                let array_length = function.dfg.type_of_value(*array).flattened_size();

                // If we are here it means the index is dynamic, so let's add a check that it's less than length
                let index = function.dfg.insert_instruction_and_results(
                    Instruction::Cast(*index, Type::unsigned(SSA_WORD_SIZE)),
                    block_id,
                    None,
                    call_stack.clone(),
                );
                let index = index.first();

                let array_typ = Type::unsigned(SSA_WORD_SIZE);
                let array_length =
                    function.dfg.make_constant((array_length as u128).into(), array_typ);
                let is_index_out_of_bounds = function.dfg.insert_instruction_and_results(
                    Instruction::binary(BinaryOp::Lt, index, array_length),
                    block_id,
                    None,
                    call_stack.clone(),
                );
                let is_index_out_of_bounds = is_index_out_of_bounds.first();
                let true_const = function.dfg.make_constant(true.into(), Type::bool());
                (is_index_out_of_bounds, true_const)
            };

            let (lhs, rhs) = apply_side_effects(
                side_effects_condition,
                lhs,
                rhs,
                function,
                block_id,
                call_stack.clone(),
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

fn instruction_might_result_in_out_of_bounds(
    function: &Function,
    instruction: &Instruction,
) -> bool {
    use Instruction::*;
    match instruction {
        ArrayGet { array, index } | ArraySet { array, index, .. } => {
            if function.dfg.try_get_array_length(*array).is_some() {
                if let Some(known_index) = function.dfg.get_numeric_constant(*index) {
                    // `index` will be relative to the flattened array length, so we need to take that into account
                    let typ = function.dfg.type_of_value(*array);
                    let array_length = typ.flattened_size();
                    known_index >= array_length.into()
                } else {
                    // A dynamic index might always be out of bounds
                    true
                }
            } else {
                // Slice operations might be out of bounds, but there's no way we
                // can insert a check because we don't know a slice's length
                false
            }
        }
        _ => false,
    }
}

fn handle_array_get_group(
    function: &Function,
    array: &ValueId,
    index: usize,
    next_out_of_bounds_index: &mut Option<usize>,
    possible_index_out_of_bounds_indexes: &mut Vec<usize>,
) {
    let Some(array_length) = function.dfg.try_get_array_length(*array) else {
        // Nothing to do for slices
        return;
    };

    let flattened_size = function.dfg.type_of_value(*array).flattened_size();
    let element_size = flattened_size / array_length;
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
    call_stack: Vector<Location>,
) -> (ValueId, ValueId) {
    // See if there's an active "enable side effects" condition
    let Some(condition) = side_effects_condition else {
        return (lhs, rhs);
    };

    let dfg = &mut function.dfg;

    // Condition needs to be cast to argument type in order to multiply them together.
    // In our case, lhs is always a boolean.
    let casted_condition = dfg.insert_instruction_and_results(
        Instruction::Cast(condition, Type::bool()),
        block_id,
        None,
        call_stack.clone(),
    );
    let casted_condition = casted_condition.first();

    let lhs = dfg.insert_instruction_and_results(
        Instruction::binary(BinaryOp::Mul, lhs, casted_condition),
        block_id,
        None,
        call_stack.clone(),
    );
    let lhs = lhs.first();

    let rhs = dfg.insert_instruction_and_results(
        Instruction::binary(BinaryOp::Mul, rhs, casted_condition),
        block_id,
        None,
        call_stack,
    );
    let rhs = rhs.first();

    (lhs, rhs)
}

#[cfg(test)]
mod test {
    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            instruction::{BinaryOp, Intrinsic},
            map::Id,
            types::Type,
        },
    };

    #[test]
    fn dead_instruction_elimination() {
        // fn main f0 {
        //   b0(v0: Field):
        //     v1 = add v0, Field 1
        //     v2 = add v0, Field 2
        //     jmp b1(v2)
        //   b1(v3: Field):
        //     v4 = allocate 1 field
        //     v5 = load v4
        //     v6 = allocate 1 field
        //     store Field 1 in v6
        //     v7 = load v6
        //     v8 = add v7, Field 1
        //     v9 = add v7, Field 2
        //     v10 = add v7, Field 3
        //     v11 = add v10, v10
        //     call assert_constant(v8)
        //     return v9
        // }
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::field());
        let b1 = builder.insert_block();

        let one = builder.field_constant(1u128);
        let two = builder.field_constant(2u128);
        let three = builder.field_constant(3u128);

        let _v1 = builder.insert_binary(v0, BinaryOp::Add, one);
        let v2 = builder.insert_binary(v0, BinaryOp::Add, two);
        builder.terminate_with_jmp(b1, vec![v2]);

        builder.switch_to_block(b1);
        let _v3 = builder.add_block_parameter(b1, Type::field());

        let v4 = builder.insert_allocate(Type::field());
        let _v5 = builder.insert_load(v4, Type::field());

        let v6 = builder.insert_allocate(Type::field());
        builder.insert_store(v6, one);
        let v7 = builder.insert_load(v6, Type::field());
        let v8 = builder.insert_binary(v7, BinaryOp::Add, one);
        let v9 = builder.insert_binary(v7, BinaryOp::Add, two);
        let v10 = builder.insert_binary(v7, BinaryOp::Add, three);
        let _v11 = builder.insert_binary(v10, BinaryOp::Add, v10);

        let assert_constant_id = builder.import_intrinsic_id(Intrinsic::AssertConstant);
        builder.insert_call(assert_constant_id, vec![v8], vec![]);
        builder.terminate_with_return(vec![v9]);

        let ssa = builder.finish();
        let main = ssa.main();

        // The instruction count never includes the terminator instruction
        assert_eq!(main.dfg[main.entry_block()].instructions().len(), 2);
        assert_eq!(main.dfg[b1].instructions().len(), 10);

        // Expected output:
        //
        // fn main f0 {
        //   b0(v0: Field):
        //     v2 = add v0, Field 2
        //     jmp b1(v2)
        //   b1(v3: Field):
        //     v6 = allocate 1 field
        //     store Field 1 in v6
        //     v7 = load v6
        //     v8 = add v7, Field 1
        //     v9 = add v7, Field 2
        //     call assert_constant(v8)
        //     return v9
        // }
        let ssa = ssa.dead_instruction_elimination();
        let main = ssa.main();

        assert_eq!(main.dfg[main.entry_block()].instructions().len(), 1);
        assert_eq!(main.dfg[b1].instructions().len(), 6);
    }

    #[test]
    fn as_witness_die() {
        // fn main f0 {
        //   b0(v0: Field):
        //     v1 = add v0, Field 1
        //     v2 = add v0, Field 2
        //     call as_witness(v2)
        //     return v1
        // }
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::field());

        let one = builder.field_constant(1u128);
        let two = builder.field_constant(2u128);

        let v1 = builder.insert_binary(v0, BinaryOp::Add, one);
        let v2 = builder.insert_binary(v0, BinaryOp::Add, two);
        let as_witness = builder.import_intrinsic("as_witness").unwrap();
        builder.insert_call(as_witness, vec![v2], Vec::new());
        builder.terminate_with_return(vec![v1]);

        let ssa = builder.finish();
        let main = ssa.main();

        // The instruction count never includes the terminator instruction
        assert_eq!(main.dfg[main.entry_block()].instructions().len(), 3);

        // Expected output:
        //
        // acir(inline) fn main f0 {
        //    b0(v0: Field):
        //      v3 = add v0, Field 1
        //      return v3
        //  }
        let ssa = ssa.dead_instruction_elimination();
        let main = ssa.main();

        assert_eq!(main.dfg[main.entry_block()].instructions().len(), 1);
    }
}
