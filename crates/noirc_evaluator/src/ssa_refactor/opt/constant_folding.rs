use std::collections::{HashMap, HashSet};

use iter_extended::vecmap;

use crate::ssa_refactor::{
    ir::{
        basic_block::BasicBlockId,
        dfg::InsertInstructionResult,
        function::Function,
        instruction::InstructionId,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Performs constant folding on each instruction.
    ///
    /// This is generally done automatically but this pass can become needed
    /// if `DataFlowGraph::set_value` or `DataFlowGraph::set_value_from_id` are
    /// used on a value which enables instructions dependent on the value to
    /// now be simplified.
    pub(crate) fn fold_constants(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            constant_fold(function);
        }
        self
    }
}

/// The structure of this pass is simple:
/// Go through each block and re-insert all instructions.
fn constant_fold(function: &mut Function) {
    let mut context = Context::default();
    context.block_queue.push(function.entry_block());

    while let Some(block) = context.block_queue.pop() {
        if context.visited_blocks.contains(&block) {
            continue;
        }

        context.visited_blocks.insert(block);
        context.fold_constants_in_block(function, block);
    }
}

#[derive(Default)]
struct Context {
    /// Maps pre-folded ValueIds to the new ValueIds obtained by re-inserting the instruction.
    values: HashMap<ValueId, ValueId>,
    visited_blocks: HashSet<BasicBlockId>,
    block_queue: Vec<BasicBlockId>,
}

impl Context {
    fn fold_constants_in_block(&mut self, function: &mut Function, block: BasicBlockId) {
        let instructions = std::mem::take(function.dfg[block].instructions_mut());

        for instruction in instructions {
            self.push_instruction(function, block, instruction);
        }

        let terminator = function.dfg[block]
            .unwrap_terminator()
            .clone()
            .map_values(|value| self.resolve_value(function, value));

        function.dfg.set_block_terminator(block, terminator);
        self.block_queue.extend(function.dfg[block].successors());
    }

    /// Fetches the folded version of the given `ValueId`. If the value refers to an instruction
    /// result, the folded value will have already been inserted into the cache. If it is an array
    /// and it isn't already in the cache, it will be recursively resolved and added to the cache.
    /// This is necessary because processing instruction results alone is not sufficient for
    /// catching all arrays that have been affected by the pass.
    fn resolve_value(&mut self, function: &mut Function, value: ValueId) -> ValueId {
        if let Some(folded_value) = self.values.get(&value) {
            return *folded_value;
        }
        let old_value = function.dfg[value].clone();
        match old_value {
            Value::Array { array, element_type } => {
                let new_array =
                    array.into_iter().map(|elem| self.resolve_value(function, elem)).collect();

                let new_value_id = function.dfg.make_array(new_array, element_type);
                self.values.insert(value, new_value_id);
                new_value_id
            }
            Value::Instruction { .. } => {
                unreachable!(
                    "Each instruction is folded and pushed before its result is referenced"
                )
            }
            Value::Function(_)
            | Value::Intrinsic(_)
            | Value::NumericConstant { .. }
            | Value::Param { .. } => {
                // These values are unaffected by instruction folding
                value
            }
        }
    }

    fn push_instruction(
        &mut self,
        function: &mut Function,
        block: BasicBlockId,
        id: InstructionId,
    ) {
        let instruction =
            function.dfg[id].clone().map_values(|id| self.resolve_value(function, id));
        let results = function.dfg.instruction_results(id).to_vec();

        let ctrl_typevars = instruction
            .requires_ctrl_typevars()
            .then(|| vecmap(&results, |result| function.dfg.type_of_value(*result)));

        let new_results =
            function.dfg.insert_instruction_and_results(instruction, block, ctrl_typevars);

        Self::insert_new_instruction_results(&mut self.values, &results, new_results);
    }

    /// Modify the values HashMap to remember the mapping between an instruction result's previous
    /// ValueId (from the source_function) and its new ValueId in the destination function.
    fn insert_new_instruction_results(
        values: &mut HashMap<ValueId, ValueId>,
        old_results: &[ValueId],
        new_results: InsertInstructionResult,
    ) {
        assert_eq!(old_results.len(), new_results.len());

        match new_results {
            InsertInstructionResult::SimplifiedTo(new_result) => {
                values.insert(old_results[0], new_result);
            }
            InsertInstructionResult::Results(new_results) => {
                for (old_result, new_result) in old_results.iter().zip(new_results) {
                    values.insert(*old_result, *new_result);
                }
            }
            InsertInstructionResult::InstructionRemoved => (),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ssa_refactor::{
        ir::{
            function::RuntimeType,
            instruction::{BinaryOp, TerminatorInstruction},
            map::Id,
            types::Type,
            value::Value,
        },
        ssa_builder::FunctionBuilder,
    };

    #[test]
    fn simple_constant_fold() {
        // fn main f0 {
        //   b0(v0: Field):
        //     v1 = add v0, Field 1
        //     v2 = mul v1, Field 3
        //     return v2
        // }
        //
        // After constructing this IR, we set the value of v0 to 2.
        // The expected return afterwards should be 9.
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id, RuntimeType::Acir);
        let v0 = builder.add_parameter(Type::field());

        let one = builder.field_constant(1u128);
        let two = builder.field_constant(2u128);
        let three = builder.field_constant(3u128);

        let v1 = builder.insert_binary(v0, BinaryOp::Add, one);
        let v2 = builder.insert_binary(v1, BinaryOp::Mul, three);
        builder.terminate_with_return(vec![v2]);

        let mut ssa = builder.finish();
        let main = ssa.main_mut();
        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 2); // The final return is not counted

        // Expected output:
        //
        // fn main f0 {
        //   b0(Field 2: Field):
        //     return Field 9
        // }
        main.dfg.set_value_from_id(v0, two);

        let ssa = ssa.fold_constants();
        let main = ssa.main();
        let block = &main.dfg[main.entry_block()];
        assert_eq!(block.instructions().len(), 0);

        match block.terminator() {
            Some(TerminatorInstruction::Return { return_values }) => {
                let value = main
                    .dfg
                    .get_numeric_constant(return_values[0])
                    .expect("Expected constant 9")
                    .to_u128();
                assert_eq!(value, 9);
            }
            _ => unreachable!("b0 should have a return terminator"),
        }
    }

    #[test]
    fn arrays_elements_are_updated() {
        // fn main f0 {
        //   b0(v0: Field):
        //     v1 = add v0, Field 1
        //     return [v1]
        // }
        //
        // After constructing this IR, we run constant folding with no expected benefit, but to
        // ensure that all new values ids are correctly propagated.
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id, RuntimeType::Acir);
        let v0 = builder.add_parameter(Type::field());
        let one = builder.field_constant(1u128);
        let v1 = builder.insert_binary(v0, BinaryOp::Add, one);
        let arr =
            builder.current_function.dfg.make_array(vec![v1].into(), vec![Type::field()].into());
        builder.terminate_with_return(vec![arr]);

        let ssa = builder.finish().fold_constants();
        let main = ssa.main();
        let entry_block_id = main.entry_block();
        let entry_block = &main.dfg[entry_block_id];
        assert_eq!(entry_block.instructions().len(), 1);
        let new_add_instr = entry_block.instructions().first().unwrap();
        let new_add_instr_result = main.dfg.instruction_results(*new_add_instr)[0];
        assert_ne!(new_add_instr_result, v1);

        let return_value_id = match entry_block.unwrap_terminator() {
            TerminatorInstruction::Return { return_values } => return_values[0],
            _ => unreachable!(),
        };
        let return_element = match &main.dfg[return_value_id] {
            Value::Array { array, .. } => array[0],
            _ => unreachable!(),
        };
        // The return element is expected to refer to the new add instruction result rather than
        // the old.
        assert_eq!(new_add_instr_result, return_element);
        assert_ne!(v1, return_element);
    }
}
