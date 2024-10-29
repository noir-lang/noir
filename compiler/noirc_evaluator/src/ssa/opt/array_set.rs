use std::mem;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        function::{Function, RuntimeType},
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

        let mut context =
            Context::new(&self.dfg, matches!(self.runtime(), RuntimeType::Brillig(_)));

        for block in reachable_blocks.iter() {
            context.analyze_last_uses(*block);
        }

        let instructions_to_update = mem::take(&mut context.instructions_that_can_be_made_mutable);
        for block in reachable_blocks {
            make_mutable(&mut self.dfg, block, &instructions_to_update);
        }
    }
}

struct Context<'f> {
    dfg: &'f DataFlowGraph,
    is_brillig_runtime: bool,
    array_to_last_use: HashMap<ValueId, InstructionId>,
    instructions_that_can_be_made_mutable: HashSet<InstructionId>,
    // Mapping of an array that comes from a load and whether the address
    // it was loaded from is a reference parameter.
    arrays_from_load: HashMap<ValueId, bool>,
    inner_nested_arrays: HashMap<ValueId, InstructionId>,
}

impl<'f> Context<'f> {
    fn new(dfg: &'f DataFlowGraph, is_brillig_runtime: bool) -> Self {
        Context {
            dfg,
            is_brillig_runtime,
            array_to_last_use: HashMap::default(),
            instructions_that_can_be_made_mutable: HashSet::default(),
            arrays_from_load: HashMap::default(),
            inner_nested_arrays: HashMap::default(),
        }
    }

    /// Builds the set of ArraySet instructions that can be made mutable
    /// because their input value is unused elsewhere afterward.
    fn analyze_last_uses(&mut self, block_id: BasicBlockId) {
        let block = &self.dfg[block_id];

        for instruction_id in block.instructions() {
            match &self.dfg[*instruction_id] {
                Instruction::ArrayGet { array, .. } => {
                    let array = self.dfg.resolve(*array);

                    if let Some(existing) = self.array_to_last_use.insert(array, *instruction_id) {
                        self.instructions_that_can_be_made_mutable.remove(&existing);
                    }
                }
                Instruction::ArraySet { array, value, .. } => {
                    let array = self.dfg.resolve(*array);

                    if let Some(existing) = self.array_to_last_use.insert(array, *instruction_id) {
                        self.instructions_that_can_be_made_mutable.remove(&existing);
                    }
                    if self.is_brillig_runtime {
                        let value = self.dfg.resolve(*value);

                        if let Some(existing) = self.inner_nested_arrays.get(&value) {
                            self.instructions_that_can_be_made_mutable.remove(existing);
                        }
                        let result = self.dfg.instruction_results(*instruction_id)[0];
                        self.inner_nested_arrays.insert(result, *instruction_id);
                    }

                    // If the array we are setting does not come from a load we can safely mark it mutable.
                    // If the array comes from a load we may potentially being mutating an array at a reference
                    // that is loaded from by other values.
                    let terminator = self.dfg[block_id].unwrap_terminator();
                    // If we are in a return block we are not concerned about the array potentially being mutated again.
                    let is_return_block =
                        matches!(terminator, TerminatorInstruction::Return { .. });
                    // We also want to check that the array is not part of the terminator arguments, as this means it is used again.
                    let mut array_in_terminator = false;
                    terminator.for_each_value(|value| {
                        if value == array {
                            array_in_terminator = true;
                        }
                    });
                    if let Some(is_from_param) = self.arrays_from_load.get(&array) {
                        // If the array was loaded from a reference parameter, we cannot
                        // safely mark that array mutable as it may be shared by another value.
                        if !is_from_param && is_return_block {
                            self.instructions_that_can_be_made_mutable.insert(*instruction_id);
                        }
                    } else if !array_in_terminator {
                        self.instructions_that_can_be_made_mutable.insert(*instruction_id);
                    }
                }
                Instruction::Call { arguments, .. } => {
                    for argument in arguments {
                        if matches!(self.dfg.type_of_value(*argument), Array { .. } | Slice { .. })
                        {
                            let argument = self.dfg.resolve(*argument);

                            if let Some(existing) =
                                self.array_to_last_use.insert(argument, *instruction_id)
                            {
                                self.instructions_that_can_be_made_mutable.remove(&existing);
                            }
                        }
                    }
                }
                Instruction::Load { address } => {
                    let result = self.dfg.instruction_results(*instruction_id)[0];
                    if matches!(self.dfg.type_of_value(result), Array { .. } | Slice { .. }) {
                        let is_reference_param =
                            self.dfg.block_parameters(block_id).contains(address);
                        self.arrays_from_load.insert(result, is_reference_param);
                    }
                }
                _ => (),
            }
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
    use noirc_frontend::monomorphization::ast::InlineType;

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
        // brillig fn main f0 {
        //     b0():
        //       v3 = allocate
        //       store [[Field 0, Field 0, Field 0, Field 0, Field 0], [Field 0, Field 0, Field 0, Field 0, Field 0]] at v3
        //       v4 = allocate
        //       store [[Field 0, Field 0, Field 0, Field 0, Field 0], [Field 0, Field 0, Field 0, Field 0, Field 0]] at v4
        //       jmp b1(u32 0)
        //     b1(v6: u32):
        //       v8 = lt v6, u32 5
        //       jmpif v8 then: b3, else: b2
        //     b3():
        //       v9 = eq v6, u32 5
        //       jmpif v9 then: b4, else: b5
        //     b4():
        //       v10 = load v3
        //       store v10 at v4
        //       jmp b5()
        //     b5():
        //       v11 = load v3
        //       v13 = array_get v11, index Field 0
        //       v14 = array_set v13, index v6, value Field 20
        //       v15 = array_set v11, index v6, value v14
        //       store v15 at v3
        //       v17 = add v6, u32 1
        //       jmp b1(v17)
        //     b2():
        //       return
        //   }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Brillig(InlineType::default()));

        let array_type = Type::Array(Arc::new(vec![Type::field()]), 5);
        let zero = builder.field_constant(0u128);
        let array_constant =
            builder.array_constant(vector![zero, zero, zero, zero, zero], array_type.clone());
        let nested_array_type = Type::Array(Arc::new(vec![array_type.clone()]), 2);
        let nested_array_constant = builder
            .array_constant(vector![array_constant, array_constant], nested_array_type.clone());

        let v3 = builder.insert_allocate(array_type.clone());

        builder.insert_store(v3, nested_array_constant);

        let v4 = builder.insert_allocate(array_type.clone());
        builder.insert_store(v4, nested_array_constant);

        let b1 = builder.insert_block();
        let zero_u32 = builder.numeric_constant(0u128, Type::unsigned(32));
        builder.terminate_with_jmp(b1, vec![zero_u32]);

        // Loop header
        builder.switch_to_block(b1);
        let v5 = builder.add_block_parameter(b1, Type::unsigned(32));
        let five = builder.numeric_constant(5u128, Type::unsigned(32));
        let v8 = builder.insert_binary(v5, BinaryOp::Lt, five);

        let b2 = builder.insert_block();
        let b3 = builder.insert_block();
        let b4 = builder.insert_block();
        let b5 = builder.insert_block();
        builder.terminate_with_jmpif(v8, b3, b2);

        // Loop body
        // b3 is the if statement conditional
        builder.switch_to_block(b3);
        let two = builder.numeric_constant(5u128, Type::unsigned(32));
        let v9 = builder.insert_binary(v5, BinaryOp::Eq, two);
        builder.terminate_with_jmpif(v9, b4, b5);

        // b4 is the rest of the loop after the if statement
        builder.switch_to_block(b4);
        let v10 = builder.insert_load(v3, nested_array_type.clone());
        builder.insert_store(v4, v10);
        builder.terminate_with_jmp(b5, vec![]);

        builder.switch_to_block(b5);
        let v11 = builder.insert_load(v3, nested_array_type.clone());
        let twenty = builder.field_constant(20u128);
        let v13 = builder.insert_array_get(v11, zero, array_type.clone());
        let v14 = builder.insert_array_set(v13, v5, twenty);
        let v15 = builder.insert_array_set(v11, v5, v14);

        builder.insert_store(v3, v15);
        let one = builder.numeric_constant(1u128, Type::unsigned(32));
        let v17 = builder.insert_binary(v5, BinaryOp::Add, one);
        builder.terminate_with_jmp(b1, vec![v17]);

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

        assert_eq!(array_set_instructions.len(), 2);
        if let Instruction::ArraySet { mutable, .. } = &main.dfg[*array_set_instructions[0]] {
            // The single array set should not be marked mutable
            assert!(!mutable);
        }
    }
}
