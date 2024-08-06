//! Dead Instruction Elimination (DIE) pass: Removes any instruction without side-effects for
//! which the results are unused.
use std::collections::HashSet;

use crate::ssa::{
    ir::{
        basic_block::{BasicBlock, BasicBlockId},
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Binary, BinaryOp, Instruction, InstructionId, Intrinsic},
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
            dead_instruction_elimination(function);
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
fn dead_instruction_elimination(function: &mut Function) {
    let mut context = Context::default();
    for call_data in &function.dfg.data_bus.call_data {
        context.mark_used_instruction_results(&function.dfg, call_data.array_id);
    }

    let blocks = PostOrder::with_function(function);

    for block in blocks.as_slice() {
        context.remove_unused_instructions_in_block(function, *block);
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
    fn remove_unused_instructions_in_block(
        &mut self,
        function: &mut Function,
        block_id: BasicBlockId,
    ) {
        self.remove_unused_instructions_in_block_impl(function, block_id, true);
    }

    fn remove_unused_instructions_in_block_impl(
        &mut self,
        function: &mut Function,
        block_id: BasicBlockId,
        analyze_index_out_of_bounds: bool,
    ) {
        // Clear state because this might have been called recursively
        self.used_values.clear();
        self.instructions_to_remove.clear();
        self.rc_instructions.clear();

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

                if analyze_index_out_of_bounds
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
            self.insert_out_of_bounds_checks(
                function,
                block_id,
                &mut possible_index_out_of_bounds_indexes,
            );

            self.remove_unused_instructions_in_block_impl(function, block_id, false);
            return;
        }

        function.dfg[block_id]
            .instructions_mut()
            .retain(|instruction| !self.instructions_to_remove.contains(instruction));
    }

    fn instruction_might_result_in_out_of_bounds(
        function: Function,
        instruction: &Instruction,
    ) -> bool {
        use Instruction::*;
        match instruction {
            ArrayGet { array, index } | ArraySet { array, index, .. } => {
                if let Some(array_length) = function.dfg.try_get_array_length(*array) {
                    if let Some(known_index) = function.dfg.get_numeric_constant(*index) {
                        // If the index is known at compile-time, we can only remove it if it's not out of bounds.
                        // If it's out of bounds we'd like that to keep failing at runtime.
                        known_index >= array_length.into()
                    } else {
                        // If the index is not known at compile-time we can't remove this instruction as this
                        // might be an index out of bounds.
                        true
                    }
                } else if let ArrayGet { .. } = instruction {
                    // array_get on a slice always does an index in bounds check,
                    // so we can remove this instruction if it's unused
                    false
                } else {
                    // The same check isn't done on array_set, though
                    true
                }
            }
            _ => false,
        }
    }

    fn insert_out_of_bounds_checks(
        &mut self,
        function: &mut Function,
        block_id: BasicBlockId,
        possible_index_out_of_bounds_indexes: &mut Vec<usize>,
    ) {
        let mut next_out_of_bounds_index = possible_index_out_of_bounds_indexes.pop();

        let instructions = function.dfg[block_id].take_instructions();
        for (index, instruction_id) in instructions.iter().enumerate() {
            let instruction_id = *instruction_id;

            if let Some(out_of_bounds_index) = next_out_of_bounds_index {
                if index == out_of_bounds_index {
                    // Need to add a constrain
                    let instruction = &function.dfg[instruction_id];
                    let call_stack = function.dfg.get_call_stack(instruction_id);

                    match instruction {
                        Instruction::ArrayGet { array, index }
                        | Instruction::ArraySet { array, index, .. } => {
                            if let Some(array_length) = function.dfg.try_get_array_length(*array) {
                                if function.dfg.get_numeric_constant(*index).is_some() {
                                    // If we are here it means the index is known but out of bounds. That's always an error!
                                    let false_const =
                                        function.dfg.make_constant(false.into(), Type::bool());
                                    let true_const =
                                        function.dfg.make_constant(true.into(), Type::bool());

                                    function.dfg.insert_instruction_and_results(
                                        Instruction::Constrain(
                                            false_const,
                                            true_const,
                                            Some("Index out of bounds".to_owned().into()),
                                        ),
                                        block_id,
                                        None,
                                        call_stack,
                                    );
                                } else {
                                    // If we are here it means the index is dynamic, so let's add a check that it's less than length
                                    let index = function
                                        .dfg
                                        .insert_instruction_and_results(
                                            Instruction::Cast(
                                                *index,
                                                Type::unsigned(SSA_WORD_SIZE),
                                            ),
                                            block_id,
                                            None,
                                            call_stack.clone(),
                                        )
                                        .first();
                                    let array_length = function.dfg.make_constant(
                                        (array_length as u128).into(),
                                        Type::unsigned(SSA_WORD_SIZE),
                                    );
                                    let is_index_out_of_bounds = function
                                        .dfg
                                        .insert_instruction_and_results(
                                            Instruction::Binary(Binary {
                                                operator: BinaryOp::Lt,
                                                lhs: index,
                                                rhs: array_length,
                                            }),
                                            block_id,
                                            None,
                                            call_stack.clone(),
                                        )
                                        .first();
                                    let true_const =
                                        function.dfg.make_constant(true.into(), Type::bool());

                                    function.dfg.insert_instruction_and_results(
                                        Instruction::Constrain(
                                            is_index_out_of_bounds,
                                            true_const,
                                            Some("Index out of bounds".to_owned().into()),
                                        ),
                                        block_id,
                                        None,
                                        call_stack,
                                    );
                                }
                            } else {
                                // TODO: this is tricky because we don't know the slice length... 🤔
                            }
                        }
                        _ => panic!("Expected an ArrayGet or ArraySet instruction here"),
                    }

                    next_out_of_bounds_index = possible_index_out_of_bounds_indexes.pop();
                }
            }

            function.dfg[block_id].instructions_mut().push(instruction_id);
        }
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
}

fn instruction_might_result_in_out_of_bounds(
    function: &Function,
    instruction: &Instruction,
) -> bool {
    use Instruction::*;
    match instruction {
        ArrayGet { array, index } | ArraySet { array, index, .. } => {
            if let Some(array_length) = function.dfg.try_get_array_length(*array) {
                if let Some(known_index) = function.dfg.get_numeric_constant(*index) {
                    // If the index is known at compile-time, we can only remove it if it's not out of bounds.
                    // If it's out of bounds we'd like that to keep failing at runtime.
                    known_index >= array_length.into()
                } else {
                    // If the index is not known at compile-time we can't remove this instruction as this
                    // might be an index out of bounds.
                    true
                }
            } else if let ArrayGet { .. } = instruction {
                // array_get on a slice always does an index in bounds check,
                // so we can remove this instruction if it's unused
                false
            } else {
                // The same check isn't done on array_set, though
                true
            }
        }
        _ => false,
    }
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
