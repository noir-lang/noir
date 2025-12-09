//! This pass replaces instructions that aren't used anywhere in a function with `Noop`
//! to free up some memory.

use std::collections::HashSet;

use crate::ssa::{ir::function::Function, ssa_gen::Ssa};

impl Ssa {
    /// Remove instructions in all functions which aren't used in any blocks.
    pub(crate) fn remove_unused_instructions(mut self) -> Self {
        for function in self.functions.values_mut() {
            function.remove_unused_instructions();
        }
        self
    }
}

impl Function {
    /// Remove instructions which aren't used in any blocks.
    pub(crate) fn remove_unused_instructions(&mut self) {
        let mut used_instructions = HashSet::new();

        for block in self.reachable_blocks() {
            for instruction in self.dfg[block].instructions() {
                used_instructions.insert(*instruction);
            }
        }

        self.dfg.retain_instructions(|id| used_instructions.contains(&id));
    }
}
