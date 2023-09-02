use std::collections::{HashMap, HashSet};

use iter_extended::vecmap;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::{DataFlowGraph, InsertInstructionResult},
        function::Function,
        instruction::{Instruction, InstructionId},
        value::ValueId,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Performs constant folding on each instruction. This is done by two methods:
    ///
    /// 1. Re-insert each instruction in order to apply the constant folding which is done automatically
    ///    by the [`DataFlowGraph`] as new instructions are pushed.
    ///
    ///    This is generally done automatically but this pass can become needed
    ///    if `DataFlowGraph::set_value` or `DataFlowGraph::set_value_from_id` are
    ///    used on a value which enables instructions dependent on the value to
    ///    now be simplified.
    ///
    /// 2. Check for the existence of [pure instructions][Instruction::is_pure()] which have a duplicate earlier in the block.
    ///    These can be replaced with the results of this previous instruction.
    ///
    ///    This is only performed within this pass and so is needed when different blocks are merged,
    ///    i.e. after the [`flatten_cfg`][super::flatten_cfg] pass.
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
    visited_blocks: HashSet<BasicBlockId>,
    block_queue: Vec<BasicBlockId>,
}

impl Context {
    fn fold_constants_in_block(&mut self, function: &mut Function, block: BasicBlockId) {
        let instructions = function.dfg[block].take_instructions();

        // Cache of instructions without any side-effects along with their outputs.
        let mut cached_instruction_results: HashMap<Instruction, Vec<ValueId>> = HashMap::new();

        for instruction_id in instructions {
            Self::fold_constants_into_instruction(
                &mut function.dfg,
                block,
                instruction_id,
                &mut cached_instruction_results,
            );
        }
        self.block_queue.extend(function.dfg[block].successors());
    }

    fn fold_constants_into_instruction(
        dfg: &mut DataFlowGraph,
        block: BasicBlockId,
        id: InstructionId,
        instruction_result_cache: &mut HashMap<Instruction, Vec<ValueId>>,
    ) {
        let instruction = Self::resolve_instruction(id, dfg);
        let old_results = dfg.instruction_results(id).to_vec();

        // If a copy of this instruction exists earlier in the block, then reuse the previous results.
        if let Some(cached_results) = instruction_result_cache.get(&instruction) {
            Self::replace_result_ids(dfg, &old_results, cached_results);
            return;
        }

        // Otherwise, try inserting the instruction again to apply any optimizations using the newly resolved inputs.
        let new_results = Self::push_instruction(id, instruction.clone(), &old_results, block, dfg);

        // If the instruction is pure then we cache the results so we can reuse them if
        // the same instruction appears again later in the block.
        if instruction.is_pure(dfg) {
            instruction_result_cache.insert(instruction, new_results.clone());
        }
        Self::replace_result_ids(dfg, &old_results, &new_results);
    }

    /// Fetches an [`Instruction`] by its [`InstructionId`] and fully resolves its inputs.
    fn resolve_instruction(instruction_id: InstructionId, dfg: &DataFlowGraph) -> Instruction {
        let instruction = dfg[instruction_id].clone();

        // Resolve any inputs to ensure that we're comparing like-for-like instructions.
        instruction.map_values(|value_id| dfg.resolve(value_id))
    }

    /// Pushes a new [`Instruction`] into the [`DataFlowGraph`] which applies any optimizations
    /// based on newly resolved values for its inputs.
    ///
    /// This may result in the [`Instruction`] being optimized away or replaced with a constant value.
    fn push_instruction(
        id: InstructionId,
        instruction: Instruction,
        old_results: &[ValueId],
        block: BasicBlockId,
        dfg: &mut DataFlowGraph,
    ) -> Vec<ValueId> {
        let ctrl_typevars = instruction
            .requires_ctrl_typevars()
            .then(|| vecmap(old_results, |result| dfg.type_of_value(*result)));

        let call_stack = dfg.get_call_stack(id);
        let new_results =
            match dfg.insert_instruction_and_results(instruction, block, ctrl_typevars, call_stack)
            {
                InsertInstructionResult::SimplifiedTo(new_result) => vec![new_result],
                InsertInstructionResult::SimplifiedToMultiple(new_results) => new_results,
                InsertInstructionResult::Results(_, new_results) => new_results.to_vec(),
                InsertInstructionResult::InstructionRemoved => vec![],
            };
        // Optimizations while inserting the instruction should not change the number of results.
        assert_eq!(old_results.len(), new_results.len());

        new_results
    }

    /// Replaces a set of [`ValueId`]s inside the [`DataFlowGraph`] with another.
    fn replace_result_ids(
        dfg: &mut DataFlowGraph,
        old_results: &[ValueId],
        new_results: &[ValueId],
    ) {
        for (old_result, new_result) in old_results.iter().zip(new_results) {
            dfg.set_value_from_id(*old_result, *new_result);
        }
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            function::RuntimeType,
            instruction::{BinaryOp, Instruction, TerminatorInstruction},
            map::Id,
            types::Type,
            value::{Value, ValueId},
        },
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

        let array_type = Type::Array(Rc::new(vec![Type::field()]), 1);
        let arr = builder.current_function.dfg.make_array(vec![v1].into(), array_type);
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
        // The return element is expected to refer to the new add instruction result.
        assert_eq!(main.dfg.resolve(new_add_instr_result), main.dfg.resolve(return_element));
    }

    #[test]
    fn instruction_deduplication() {
        // fn main f0 {
        //   b0(v0: Field):
        //     v1 = cast v0 as u32
        //     v2 = cast v0 as u32
        //     constrain v1 v2
        // }
        //
        // After constructing this IR, we run constant folding which should replace the second cast
        // with a reference to the results to the first. This then allows us to optimize away
        // the constrain instruction as both inputs are known to be equal.
        //
        // The first cast instruction is retained and will be removed in the dead instruction elimination pass.
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id, RuntimeType::Acir);
        let v0 = builder.add_parameter(Type::field());

        let v1 = builder.insert_cast(v0, Type::unsigned(32));
        let v2 = builder.insert_cast(v0, Type::unsigned(32));
        builder.insert_constrain(v1, v2);

        let mut ssa = builder.finish();
        let main = ssa.main_mut();
        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 3);

        // Expected output:
        //
        // fn main f0 {
        //   b0(v0: Field):
        //     v1 = cast v0 as u32
        // }
        let ssa = ssa.fold_constants();
        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();

        assert_eq!(instructions.len(), 1);
        let instruction = &main.dfg[instructions[0]];

        assert_eq!(instruction, &Instruction::Cast(ValueId::test_new(0), Type::unsigned(32)));
    }
}
