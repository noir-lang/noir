use crate::{
    errors::RtResult,
    ssa::ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Instruction, InstructionId},
        value::{ValueId, ValueMapping},
    },
};

impl Function {
    /// Performs a simple optimization according to the given callback.
    ///
    /// The function's [`Function::reachable_blocks`] are traversed in turn, and instructions in those blocks
    /// are then traversed in turn. For each one, `f` will be called with a context.
    ///
    /// The current instruction will be inserted at the end of the callback given to `mutate` unless
    /// `remove_current_instruction` or `insert_current_instruction` are called.
    ///
    /// `insert_current_instruction` is useful if you need to insert new instructions after the current
    /// one, so this can be done before the callback ends.
    ///
    /// `replace_value` can be used to replace a value with another one. This substitution will be
    /// performed in all subsequent instructions.
    pub(crate) fn simple_reachable_blocks_optimization<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut SimpleOptimizationContext<'_, '_>),
    {
        self.simple_reachable_blocks_optimization_result(move |context| {
            f(context);
            Ok(())
        })
        .expect("`f` cannot error internally so this should be unreachable");
    }

    /// Similar to `simple_reachable_blocks_optimization` but traverses the blocks in pre-order.
    pub(crate) fn simple_reachable_pre_order_blocks_optimization<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut SimpleOptimizationContext<'_, '_>),
    {
        let blocks = self.reachable_pre_order_blocks().into_iter();

        self.simple_optimization_result(blocks, move |context| {
            f(context);
            Ok(())
        })
        .expect("`f` cannot error internally so this should be unreachable");
    }

    /// Performs a simple optimization according to the given callback, returning early if
    /// an error occurred.
    ///
    /// The function's [`Function::reachable_blocks`] are traversed in turn, and instructions in those blocks
    /// are then traversed in turn. For each one, `f` will be called with a context.
    ///
    /// The current instruction will be inserted at the end of the callback given to `mutate` unless
    /// `remove_current_instruction` or `insert_current_instruction` are called.
    ///
    /// `insert_current_instruction` is useful if you need to insert new instructions after the current
    /// one, so this can be done before the callback ends.
    ///
    /// `replace_value` can be used to replace a value with another one. This substitution will be
    /// performed in all subsequent instructions.
    pub(crate) fn simple_reachable_blocks_optimization_result<F>(&mut self, f: F) -> RtResult<()>
    where
        F: FnMut(&mut SimpleOptimizationContext<'_, '_>) -> RtResult<()>,
    {
        let blocks = self.reachable_blocks().into_iter();
        self.simple_optimization_result(blocks, f)
    }

    fn simple_optimization_result<F>(
        &mut self,
        blocks: impl Iterator<Item = BasicBlockId>,
        mut f: F,
    ) -> RtResult<()>
    where
        F: FnMut(&mut SimpleOptimizationContext<'_, '_>) -> RtResult<()>,
    {
        let mut values_to_replace = ValueMapping::default();

        for block_id in blocks {
            let instruction_ids = self.dfg[block_id].take_instructions();
            self.dfg[block_id].instructions_mut().reserve(instruction_ids.len());
            for instruction_id in &instruction_ids {
                let instruction_id = *instruction_id;

                if !values_to_replace.is_empty() {
                    let instruction = &mut self.dfg[instruction_id];
                    instruction.replace_values(&values_to_replace);
                }

                let mut context = SimpleOptimizationContext {
                    block_id,
                    instruction_id,
                    dfg: &mut self.dfg,
                    values_to_replace: &mut values_to_replace,
                    insert_current_instruction_at_callback_end: true,
                };
                f(&mut context)?;

                if context.insert_current_instruction_at_callback_end {
                    self.dfg[block_id].insert_instruction(instruction_id);
                }
            }

            self.dfg.replace_values_in_block_terminator(block_id, &values_to_replace);
        }

        self.dfg.data_bus.replace_values(&values_to_replace);
        Ok(())
    }
}

pub(crate) struct SimpleOptimizationContext<'dfg, 'mapping> {
    #[allow(unused)]
    pub(crate) block_id: BasicBlockId,
    pub(crate) instruction_id: InstructionId,
    pub(crate) dfg: &'dfg mut DataFlowGraph,
    values_to_replace: &'mapping mut ValueMapping,
    insert_current_instruction_at_callback_end: bool,
}

impl SimpleOptimizationContext<'_, '_> {
    /// Returns the current instruction being visited.
    pub(crate) fn instruction(&self) -> &Instruction {
        &self.dfg[self.instruction_id]
    }

    /// Instructs this context to replace a value with another value. The value will be replaced
    /// in all subsequent instructions.
    pub(crate) fn replace_value(&mut self, from: ValueId, to: ValueId) {
        self.values_to_replace.insert(from, to);
    }

    /// Instructs this context to insert the current instruction right away, as opposed
    /// to doing this at the end of `mutate`'s block (unless `remove_current_instruction is called`).
    pub(crate) fn insert_current_instruction(&mut self) {
        self.dfg[self.block_id].insert_instruction(self.instruction_id);
        self.insert_current_instruction_at_callback_end = false;
    }

    /// Instructs this context to remove the current instruction from its block.
    pub(crate) fn remove_current_instruction(&mut self) {
        self.insert_current_instruction_at_callback_end = false;
    }

    /// Inserts an instruction in the current block right away.
    pub(crate) fn insert_instruction(&mut self, instruction_id: InstructionId) {
        self.dfg[self.block_id].insert_instruction(instruction_id);
    }

    /// Replaces the current instruction with another one.
    pub(crate) fn replace_current_instruction_with(&mut self, instruction: Instruction) {
        self.dfg[self.instruction_id] = instruction;
    }
}
