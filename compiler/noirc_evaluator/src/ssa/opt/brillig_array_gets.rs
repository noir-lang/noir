use fxhash::FxHashMap;

use crate::{brillig::brillig_ir::BRILLIG_MEMORY_ADDRESSING_BIT_SIZE, ssa::{ir::{function::Function, instruction::{BinaryOp, Instruction}, types::NumericType}, Ssa}};




impl Ssa {

    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn brillig_array_gets(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.brillig_array_gets();
        }

        self
    }
}

impl Function {
    pub(super) fn brillig_array_gets(&mut self) {
        let reachable_blocks = self.reachable_blocks();

        let one = self.dfg.make_constant(1u128.into(), NumericType::unsigned(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE));
        let mut instructions_to_update = FxHashMap::default();
        for block_id in reachable_blocks.into_iter() {
            // let block = &self.dfg[block_id];
            for instruction_id in self.dfg[block_id].instructions().to_vec() {
                let call_stack = self.dfg.get_instruction_call_stack_id(instruction_id);
                match self.dfg[instruction_id] {
                    Instruction::ArrayGet { array, index } => {
                        // let array = *array;
                        // let index = *index;
                        // let add = Instruction::binary(BinaryOp::Add { unchecked: true }, index, one);
                        // let index = self.dfg.insert_instruction_and_results(add, block_id, None, call_stack).first();
                        // *index = result
                        instructions_to_update.insert(instruction_id, (Instruction::ArrayGet { array, index }, block_id));
                    }
                    _ => {},
                }
            }
        }

        for (instruction_id, (array_get, block_id)) in instructions_to_update {
            // match array_get {
            //     Instruction::ArrayGet { array, index }
            // }
            let call_stack = self.dfg.get_instruction_call_stack_id(instruction_id);
            let new_instruction = match self.dfg[instruction_id] {
                Instruction::ArrayGet { array, index } => {
                    let add = Instruction::binary(BinaryOp::Add { unchecked: true }, index, one);
                    let index = self.dfg.insert_instruction_and_results(add, block_id, None, call_stack).first();
                    Instruction::ArrayGet { array, index }
                }
                _ => {
                    continue;
                }
            };
            self.dfg[instruction_id] = new_instruction;
        }
    }
}