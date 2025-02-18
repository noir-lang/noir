use fxhash::FxHashMap;

use crate::{
    brillig::brillig_ir::BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
    ssa::{
        ir::{
            function::Function,
            instruction::{BinaryOp, Instruction},
            types::NumericType,
        },
        Ssa,
    },
};

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

        let mut instructions_to_update = FxHashMap::default();
        for block_id in reachable_blocks.into_iter() {
            for instruction_id in self.dfg[block_id].instructions().to_vec() {
                match self.dfg[instruction_id] {
                    Instruction::ArrayGet { array, index } => {
                        if self.dfg.is_constant(index) {
                            instructions_to_update.insert(
                                instruction_id,
                                (Instruction::ArrayGet { array, index }, block_id),
                            );
                        }
                    }
                    _ => {}
                }
            }
        }

        for (instruction_id, _) in instructions_to_update {
            let new_instruction = match self.dfg[instruction_id] {
                Instruction::ArrayGet { array, index } => {
                    let index_constant =
                        self.dfg.get_numeric_constant(index).expect("ICE: Expected constant index");
                    let index = self.dfg.make_constant(
                        index_constant + 1u128.into(),
                        NumericType::unsigned(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
                    );
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
