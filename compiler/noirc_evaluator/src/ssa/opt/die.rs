//! Dead Instruction Elimination (DIE) pass: Removes any instruction without side-effects for
//! which the results are unused.
use std::collections::HashSet;

use crate::ssa::{
    ir::{
        basic_block::{BasicBlock, BasicBlockId},
        dfg::DataFlowGraph,
        function::Function,
        instruction::InstructionId,
        post_order::PostOrder,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Performs Dead Instruction Elimination (DIE) to remove any instructions with
    /// unused results.
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
    let blocks = PostOrder::with_function(function);

    for block in blocks.as_slice() {
        context.remove_unused_instructions_in_block(function, *block);
    }
}

/// Per function context for tracking unused values and which instructions to remove.
#[derive(Default)]
struct Context {
    used_values: HashSet<ValueId>,
    instructions_to_remove: HashSet<InstructionId>,
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
        let block = &function.dfg[block_id];
        self.mark_terminator_values_as_used(function, block);

        for instruction in block.instructions().iter().rev() {
            if self.is_unused(*instruction, function) {
                self.instructions_to_remove.insert(*instruction);
            } else {
                let instruction = &function.dfg[*instruction];
                instruction.for_each_value(|value| {
                    self.mark_used_instruction_results(&function.dfg, value);
                });
            }
        }

        function.dfg[block_id]
            .instructions_mut()
            .retain(|instruction| !self.instructions_to_remove.contains(instruction));
    }

    /// Returns true if an instruction can be removed.
    ///
    /// An instruction can be removed as long as it has no side-effects, and none of its result
    /// values have been referenced.
    fn is_unused(&self, instruction_id: InstructionId, function: &Function) -> bool {
        let instruction = &function.dfg[instruction_id];

        if instruction.has_side_effects(&function.dfg) {
            // If the instruction has side effects we should never remove it.
            false
        } else {
            let results = function.dfg.instruction_results(instruction_id);
            results.iter().all(|result| !self.used_values.contains(result))
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
            _ => {
                // Does not comprise of any instruction results
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            function::RuntimeType,
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
        let mut builder = FunctionBuilder::new("main".into(), main_id, RuntimeType::Acir);
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

        let v4 = builder.insert_allocate();
        let _v5 = builder.insert_load(v4, Type::field());

        let v6 = builder.insert_allocate();
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
}
