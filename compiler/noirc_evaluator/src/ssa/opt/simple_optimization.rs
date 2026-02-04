//! Contains helper functions for performing SSA optimizations.

use std::hash::BuildHasher;

use iter_extended::vecmap;
use noirc_errors::call_stack::CallStackId;

use acvm::FieldElement;

use crate::{
    errors::RtResult,
    ssa::ir::{
        basic_block::BasicBlockId,
        dfg::{DataFlowGraph, InsertInstructionResult},
        function::Function,
        instruction::{Instruction, InstructionId},
        post_order::PostOrder,
        types::{NumericType, Type},
        value::{ValueId, ValueMapping},
    },
};

impl Function {
    /// Performs a simple optimization according to the given callback.
    ///
    /// The function's reverse [post order][PostOrder] are traversed in turn, and instructions in those blocks
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
    pub(crate) fn simple_optimization<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut SimpleOptimizationContext<'_, '_>),
    {
        self.simple_optimization_result(move |context| {
            f(context);
            Ok(())
        })
        .expect("`f` cannot error internally so this should be unreachable");
    }

    /// Performs a simple optimization according to the given callback, returning early if
    /// an error occurred.
    ///
    /// The function's reverse [post order][PostOrder] are traversed in turn, and instructions in those blocks
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
    pub(crate) fn simple_optimization_result<F>(&mut self, mut f: F) -> RtResult<()>
    where
        F: FnMut(&mut SimpleOptimizationContext<'_, '_>) -> RtResult<()>,
    {
        let mut values_to_replace = ValueMapping::default();
        let mut enable_side_effects =
            self.dfg.make_constant(FieldElement::from(1_u128), NumericType::bool());
        let reverse_post_order = PostOrder::with_function(self).into_vec_reverse();
        for block_id in reverse_post_order {
            let instruction_ids = self.dfg[block_id].take_instructions();
            self.dfg[block_id].instructions_mut().reserve(instruction_ids.len());
            for instruction_id in &instruction_ids {
                let instruction_id = *instruction_id;
                let instruction = &mut self.dfg[instruction_id];
                let orig_instruction_hash = rustc_hash::FxBuildHasher.hash_one(&instruction);
                if !values_to_replace.is_empty() {
                    instruction.replace_values(&values_to_replace);
                }
                if let Instruction::EnableSideEffectsIf { condition } = instruction {
                    enable_side_effects = *condition;
                }
                let call_stack_id = self.dfg.get_instruction_call_stack_id(instruction_id);
                let mut context = SimpleOptimizationContext {
                    block_id,
                    instruction_id,
                    call_stack_id,
                    dfg: &mut self.dfg,
                    values_to_replace: &mut values_to_replace,
                    insert_current_instruction_at_callback_end: true,
                    enable_side_effects,
                    orig_instruction_hash,
                };
                f(&mut context)?;

                if context.insert_current_instruction_at_callback_end {
                    context.insert_current_instruction();
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
    pub(crate) call_stack_id: CallStackId,
    pub(crate) dfg: &'dfg mut DataFlowGraph,
    pub(crate) enable_side_effects: ValueId,
    values_to_replace: &'mapping mut ValueMapping,
    insert_current_instruction_at_callback_end: bool,
    orig_instruction_hash: u64,
}

impl SimpleOptimizationContext<'_, '_> {
    /// Returns the current instruction being visited.
    ///
    /// The instruction has already had its values updated with any replacements to be done.
    pub(crate) fn instruction(&self) -> &Instruction {
        &self.dfg[self.instruction_id]
    }

    /// Instructs this context to replace a value with another value. The value will be replaced
    /// in all subsequent instructions.
    pub(crate) fn replace_value(&mut self, from: ValueId, to: ValueId) {
        self.values_to_replace.insert(from, to);
    }

    /// Check if the instruction has changed relative to its original contents,
    /// e.g. because any of its values have been replaced.
    fn has_instruction_changed(&self) -> bool {
        // If the instruction changed, then there is a chance that we can (or have to)
        // simplify it before we insert it back into the block.
        let instruction_hash = rustc_hash::FxBuildHasher.hash_one(self.instruction());
        self.orig_instruction_hash != instruction_hash
    }

    /// Instructs this context to insert the current instruction right away, as opposed
    /// to doing this at the end of `mutate`'s block (unless `remove_current_instruction is called`).
    ///
    /// If the instruction or its values has changed relative to their original content,
    /// we attempt to simplify the instruction before re-inserting it into the block.
    pub(crate) fn insert_current_instruction(&mut self) {
        // If the instruction changed, then there is a chance that we can (or have to)
        // simplify it before we insert it back into the block.
        let simplify = self.has_instruction_changed();

        if simplify {
            // Based on FunctionInserter::push_instruction_value.
            let instruction = self.instruction().clone();
            let results = self.dfg.instruction_results(self.instruction_id).to_vec();
            let ctrl_typevars = instruction
                .requires_ctrl_typevars()
                .then(|| vecmap(&results, |result| self.dfg.type_of_value(*result)));

            let new_results = self.dfg.insert_instruction_and_results_if_simplified(
                instruction,
                self.block_id,
                ctrl_typevars,
                self.call_stack_id,
                Some(self.instruction_id),
            );
            assert_eq!(results.len(), new_results.len());
            for i in 0..results.len() {
                self.values_to_replace.insert(results[i], new_results[i]);
            }
        } else {
            self.dfg[self.block_id].insert_instruction(self.instruction_id);
        }

        self.insert_current_instruction_at_callback_end = false;
    }

    /// Instructs this context to remove the current instruction from its block.
    pub(crate) fn remove_current_instruction(&mut self) {
        self.insert_current_instruction_at_callback_end = false;
    }

    /// Inserts an instruction in the current block right away.
    pub(crate) fn insert_instruction(
        &mut self,
        instruction: Instruction,
        ctrl_typevars: Option<Vec<Type>>,
    ) -> InsertInstructionResult {
        self.dfg.insert_instruction_and_results(
            instruction,
            self.block_id,
            ctrl_typevars,
            self.call_stack_id,
        )
    }

    /// Inserts an instruction by id in the current block right away.
    pub(crate) fn insert_instruction_by_id(&mut self, instruction_id: InstructionId) {
        self.dfg[self.block_id].insert_instruction(instruction_id);
    }

    /// Replaces the current instruction with another one.
    ///
    /// This assumes no change in the number and type of results,
    /// it simply reassigns the existing ID with a modified instruction.
    pub(crate) fn replace_current_instruction_with(&mut self, instruction: Instruction) {
        self.dfg[self.instruction_id] = instruction;
    }
}
