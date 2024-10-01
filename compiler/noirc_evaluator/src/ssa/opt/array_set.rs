use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Instruction, InstructionId, TerminatorInstruction},
        types::Type::{Array, Slice},
        value::ValueId,
    },
    ssa_gen::Ssa,
};
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

impl Ssa {
    /// Map arrays with the last instruction that uses it
    /// For this we simply process all the instructions in execution order
    /// and update the map whenever there is a match
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn array_set_optimization(mut self) -> Self {
        for func in self.functions.values_mut() {
            func.array_set_optimization();
        }
        self
    }
}

impl Function {
    pub(crate) fn array_set_optimization(&mut self) {
        let reachable_blocks = self.reachable_blocks();

        if !self.runtime().is_entry_point() {
            assert_eq!(reachable_blocks.len(), 1, "Expected there to be 1 block remaining in Acir function for array_set optimization");
        }
        let mut array_to_last_use = HashMap::default();
        let mut instructions_to_update = HashSet::default();
        let mut arrays_from_load = HashSet::default();

        for block in reachable_blocks.iter() {
            analyze_last_uses(
                &self.dfg,
                *block,
                &mut array_to_last_use,
                &mut instructions_to_update,
                &mut arrays_from_load,
            );
        }
        for block in reachable_blocks {
            make_mutable(&mut self.dfg, block, &instructions_to_update);
        }
    }
}

/// Builds the set of ArraySet instructions that can be made mutable
/// because their input value is unused elsewhere afterward.
fn analyze_last_uses(
    dfg: &DataFlowGraph,
    block_id: BasicBlockId,
    array_to_last_use: &mut HashMap<ValueId, InstructionId>,
    instructions_that_can_be_made_mutable: &mut HashSet<InstructionId>,
    arrays_from_load: &mut HashSet<ValueId>,
) {
    let block = &dfg[block_id];

    for instruction_id in block.instructions() {
        match &dfg[*instruction_id] {
            Instruction::ArrayGet { array, .. } => {
                let array = dfg.resolve(*array);

                if let Some(existing) = array_to_last_use.insert(array, *instruction_id) {
                    instructions_that_can_be_made_mutable.remove(&existing);
                }
            }
            Instruction::ArraySet { array, .. } => {
                let array = dfg.resolve(*array);

                if let Some(existing) = array_to_last_use.insert(array, *instruction_id) {
                    instructions_that_can_be_made_mutable.remove(&existing);
                }
                // If the array we are setting does not come from a load we can safely mark it mutable.
                // If the array comes from a load we may potentially being mutating an array at a reference
                // that is loaded from by other values.
                let terminator = dfg[block_id].unwrap_terminator();
                // If we are in a return block we are not concerned about the array potentially being mutated again.
                let is_return_block = matches!(terminator, TerminatorInstruction::Return { .. });
                // We also want to check that the array is not part of the terminator arguments, as this means it is used again.
                let mut array_in_terminator = false;
                terminator.for_each_value(|value| {
                    if value == array {
                        array_in_terminator = true;
                    }
                });
                if (!arrays_from_load.contains(&array) || is_return_block) && !array_in_terminator {
                    instructions_that_can_be_made_mutable.insert(*instruction_id);
                }
            }
            Instruction::Call { arguments, .. } => {
                for argument in arguments {
                    if matches!(dfg.type_of_value(*argument), Array { .. } | Slice { .. }) {
                        let argument = dfg.resolve(*argument);

                        if let Some(existing) = array_to_last_use.insert(argument, *instruction_id)
                        {
                            instructions_that_can_be_made_mutable.remove(&existing);
                        }
                    }
                }
            }
            Instruction::Load { .. } => {
                let result = dfg.instruction_results(*instruction_id)[0];
                if matches!(dfg.type_of_value(result), Array { .. } | Slice { .. }) {
                    arrays_from_load.insert(result);
                }
            }
            _ => (),
        }
    }
}

/// Make each ArraySet instruction in `instructions_to_update` mutable.
fn make_mutable(
    dfg: &mut DataFlowGraph,
    block_id: BasicBlockId,
    instructions_to_update: &HashSet<InstructionId>,
) {
    if instructions_to_update.is_empty() {
        return;
    }

    // Take the instructions temporarily so we can mutate the DFG while we iterate through them
    let block = &mut dfg[block_id];
    let instructions = block.take_instructions();

    for instruction in &instructions {
        if instructions_to_update.contains(instruction) {
            let instruction = &mut dfg[*instruction];

            if let Instruction::ArraySet { mutable, .. } = instruction {
                *mutable = true;
            } else {
                unreachable!(
                    "Non-ArraySet instruction in instructions_to_update!\n{instruction:?}"
                );
            }
        }
    }

    *dfg[block_id].instructions_mut() = instructions;
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use im::vector;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            function::RuntimeType,
            instruction::{BinaryOp, Instruction},
            map::Id,
            types::Type,
        },
    };

    #[test]
    fn array_set_in_loop_with_conditional_clone() {
        // We want to make sure that we do not mark a single array set mutable which is loaded
        // from and cloned in a loop. If the array is inadvertently marked mutable, and is cloned in a previous iteration
        // of the loop, its clone will also be altered.
        //
        // acir(inline) fn main f0 {
        //     b0():
        //       v2 = allocate
        //       store [Field 0, Field 0, Field 0, Field 0, Field 0] at v2
        //       v3 = allocate
        //       store [Field 0, Field 0, Field 0, Field 0, Field 0] at v3
        //       jmp b1(u32 0)
        //     b1(v5: u32):
        //       v7 = lt v5, u32 5
        //       jmpif v7 then: b3, else: b2
        //     b3():
        //       v8 = eq v5, u32 5
        //       jmpif v8 then: b4, else: b5
        //     b4():
        //       v9 = load v2
        //       store v9 at v3
        //       jmp b5()
        //     b5():
        //       v10 = load v2
        //       v12 = array_set v10, index v5, value Field 20
        //       store v12 at v2
        //       v14 = add v5, u32 1
        //       jmp b1(v14)
        //     b2():
        //       return
        //   }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Brillig);

        let array_type = Type::Array(Arc::new(vec![Type::field()]), 5);
        let zero = builder.field_constant(0u128);
        let array_constant =
            builder.array_constant(vector![zero, zero, zero, zero, zero], array_type.clone());

        let v2 = builder.insert_allocate(array_type.clone());

        builder.insert_store(v2, array_constant);

        let v3 = builder.insert_allocate(array_type.clone());
        builder.insert_store(v3, array_constant);

        let b1 = builder.insert_block();
        let zero_u32 = builder.numeric_constant(0u128, Type::unsigned(32));
        builder.terminate_with_jmp(b1, vec![zero_u32]);

        // Loop header
        builder.switch_to_block(b1);
        let v5 = builder.add_block_parameter(b1, Type::unsigned(32));
        let five = builder.numeric_constant(5u128, Type::unsigned(32));
        let v7 = builder.insert_binary(v5, BinaryOp::Lt, five);

        let b2 = builder.insert_block();
        let b3 = builder.insert_block();
        let b4 = builder.insert_block();
        let b5 = builder.insert_block();
        builder.terminate_with_jmpif(v7, b3, b2);

        // Loop body
        // b3 is the if statement conditional
        builder.switch_to_block(b3);
        let two = builder.numeric_constant(5u128, Type::unsigned(32));
        let v8 = builder.insert_binary(v5, BinaryOp::Eq, two);
        builder.terminate_with_jmpif(v8, b4, b5);

        // b4 is the rest of the loop after the if statement
        builder.switch_to_block(b4);
        let v9 = builder.insert_load(v2, array_type.clone());
        builder.insert_store(v3, v9);
        builder.terminate_with_jmp(b5, vec![]);

        builder.switch_to_block(b5);
        let v10 = builder.insert_load(v2, array_type.clone());
        let twenty = builder.field_constant(20u128);
        let v12 = builder.insert_array_set(v10, v5, twenty);
        builder.insert_store(v2, v12);
        let one = builder.numeric_constant(1u128, Type::unsigned(32));
        let v14 = builder.insert_binary(v5, BinaryOp::Add, one);
        builder.terminate_with_jmp(b1, vec![v14]);

        builder.switch_to_block(b2);
        builder.terminate_with_return(vec![]);

        let ssa = builder.finish();
        // We expect the same result as above
        let ssa = ssa.array_set_optimization();

        let main = ssa.main();
        assert_eq!(main.reachable_blocks().len(), 6);

        let array_set_instructions = main.dfg[b5]
            .instructions()
            .iter()
            .filter(|instruction| matches!(&main.dfg[**instruction], Instruction::ArraySet { .. }))
            .collect::<Vec<_>>();

        assert_eq!(array_set_instructions.len(), 1);
        if let Instruction::ArraySet { mutable, .. } = &main.dfg[*array_set_instructions[0]] {
            // The single array set should not be marked mutable
            assert!(!mutable);
        }
    }
}
