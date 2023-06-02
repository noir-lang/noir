use std::collections::{HashMap, HashSet};

use iter_extended::vecmap;

use crate::ssa_refactor::{
    ir::{
        basic_block::BasicBlockId, dfg::InsertInstructionResult, function::Function,
        instruction::InstructionId, value::ValueId,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    pub(crate) fn fold_constants(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            constant_fold(function);
        }
        self
    }
}

fn constant_fold(function: &mut Function) {
    let mut context = Context::default();
    context.block_queue.push(function.entry_block());

    while let Some(block) = context.block_queue.pop() {
        if context.visited_blocks.contains(&block) {
            continue;
        }

        context.fold_constants_in_block(function, block);
    }
}

#[derive(Default)]
struct Context {
    /// Maps pre-unrolled ValueIds to unrolled ValueIds.
    /// These will often be the exact same as before, unless the ValueId was
    /// dependent on the loop induction variable which is changing on each iteration.
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

        let terminator =
            function.dfg[block].unwrap_terminator().map_values(|value| self.get_value(value));

        function.dfg.set_block_terminator(block, terminator);
        self.block_queue.extend(function.dfg[block].successors());
    }

    fn get_value(&self, value: ValueId) -> ValueId {
        self.values.get(&value).copied().unwrap_or(value)
    }

    fn push_instruction(
        &mut self,
        function: &mut Function,
        block: BasicBlockId,
        id: InstructionId,
    ) {
        let instruction = function.dfg[id].map_values(|id| self.get_value(id));
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
