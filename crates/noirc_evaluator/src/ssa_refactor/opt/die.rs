use std::collections::HashSet;

use crate::ssa_refactor::{
    ir::{
        basic_block::{BasicBlock, BasicBlockId},
        function::Function,
        instruction::{Instruction, InstructionId},
        post_order::PostOrder,
        value::ValueId,
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

fn dead_instruction_elimination(function: &mut Function) {
    let mut context = Context::default();
    let blocks = PostOrder::with_function(function);

    for block in blocks.as_slice() {
        context.remove_unused_instructions_in_block(function, *block);
    }
}

#[derive(Default)]
struct Context {
    used_values: HashSet<ValueId>,
    instructions_to_remove: HashSet<InstructionId>,
}

impl Context {
    fn remove_unused_instructions_in_block(
        &mut self,
        function: &mut Function,
        block_id: BasicBlockId,
    ) {
        let block = &function.dfg[block_id];
        self.mark_terminator_values_as_used(block);

        for instruction in block.instructions().iter().rev() {
            if self.is_unused(*instruction, function) {
                self.instructions_to_remove.insert(*instruction);
            } else {
                let instruction = &function.dfg[*instruction];
                instruction.for_each_value(|value| self.used_values.insert(value));
            }
        }

        function.dfg[block_id]
            .instructions_mut()
            .retain(|instruction| !self.instructions_to_remove.contains(instruction));
    }

    fn is_unused(&self, instruction_id: InstructionId, function: &Function) -> bool {
        use Instruction::*;

        let instruction = &function.dfg[instruction_id];

        // These instruction types cannot be removed
        if matches!(instruction, Constrain(_) | Call { .. } | Store { .. }) {
            return false;
        }

        let results = function.dfg.instruction_results(instruction_id);
        results.iter().all(|result| !self.used_values.contains(result))
    }

    fn mark_terminator_values_as_used(&mut self, block: &BasicBlock) {
        block.unwrap_terminator().for_each_value(|value| self.used_values.insert(value));
    }
}

#[cfg(test)]
mod test {
    use crate::ssa_refactor::{
        ir::{function::RuntimeType, instruction::BinaryOp, map::Id, types::Type},
        ssa_builder::FunctionBuilder,
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
        //     call println(v8)
        //     return v9
        // }
        let main_id = Id::test_new(0);
        let println_id = Id::test_new(1);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id, RuntimeType::Acir);
        let v0 = builder.add_parameter(Type::field());
        let b1 = builder.insert_block();

        let zero = builder.field_constant(0u128);
        let one = builder.field_constant(1u128);
        let two = builder.field_constant(2u128);
        let three = builder.field_constant(3u128);

        let _v1 = builder.insert_binary(v0, BinaryOp::Add, one);
        let v2 = builder.insert_binary(v0, BinaryOp::Add, two);
        builder.terminate_with_jmp(b1, vec![v2]);

        builder.switch_to_block(b1);
        let _v3 = builder.add_block_parameter(b1, Type::field());

        let v4 = builder.insert_allocate(1);
        let _v5 = builder.insert_load(v4, zero, Type::field());

        let v6 = builder.insert_allocate(1);
        builder.insert_store(v6, one);
        let v7 = builder.insert_load(v6, zero, Type::field());
        let v8 = builder.insert_binary(v7, BinaryOp::Add, one);
        let v9 = builder.insert_binary(v7, BinaryOp::Add, two);
        let v10 = builder.insert_binary(v7, BinaryOp::Add, three);
        let _v11 = builder.insert_binary(v10, BinaryOp::Add, v10);
        builder.insert_call(println_id, vec![v8], vec![]);
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
        //     call println(v8)
        //     return v9
        // }
        let ssa = ssa.dead_instruction_elimination();
        let main = ssa.main();

        assert_eq!(main.dfg[main.entry_block()].instructions().len(), 1);
        assert_eq!(main.dfg[b1].instructions().len(), 6);
    }
}
