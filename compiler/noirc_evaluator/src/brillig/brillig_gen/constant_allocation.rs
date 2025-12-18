//! This module analyzes the usage of constants in a given function and decides an allocation point for them.
//! The allocation point will be the common dominator of all the places where the constant is used.
//! By allocating in the common dominator, we can cache the constants for all subsequent uses.

use std::collections::{BTreeMap, BTreeSet};

use crate::ssa::ir::{
    basic_block::BasicBlockId,
    dfg::DataFlowGraph,
    dom::DominatorTree,
    function::Function,
    instruction::InstructionId,
    value::{Value, ValueId},
};

use super::variable_liveness::{is_variable, variables_used_in_instruction};
use crate::ssa::opt::Loops;

/// Indicate where a variable was used in a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum InstructionLocation {
    Instruction(InstructionId),
    Terminator,
}

/// Decisions about which block a constant should be allocated in.
///
/// The allocation points can further be hoisted into the global
/// space if they are shared across functions. That algorithm is
/// based on these per-function results.
#[derive(Default)]
pub(crate) struct ConstantAllocation {
    /// Map each constant to the blocks and instructions in which it is used.
    constant_usage: BTreeMap<ValueId, BTreeMap<BasicBlockId, Vec<InstructionLocation>>>,
    /// Map each block and instruction to the vector of constants that should be allocated at that point.
    allocation_points: BTreeMap<BasicBlockId, BTreeMap<InstructionLocation, Vec<ValueId>>>,
    /// The dominator tree is used to find the common dominator of all the blocks that share a constant.
    dominator_tree: DominatorTree,
    /// Further to finding the common dominator, we try to find a dominator that isn't part of a loop.
    blocks_within_loops: BTreeSet<BasicBlockId>,
}

impl ConstantAllocation {
    /// Run the constant allocation algorithm for a [Function] and return the decisions.
    pub(crate) fn from_function(func: &Function) -> Self {
        let loops = Loops::find_all(func);
        let blocks_within_loops =
            loops.yet_to_unroll.into_iter().flat_map(|_loop| _loop.blocks).collect();

        let mut instance = ConstantAllocation {
            constant_usage: BTreeMap::default(),
            allocation_points: BTreeMap::default(),
            dominator_tree: loops.dom,
            blocks_within_loops,
        };
        instance.collect_constant_usage(func);
        instance.decide_allocation_points(func);

        instance
    }

    /// Collect all constants allocated in a given block.
    pub(crate) fn allocated_in_block(&self, block_id: BasicBlockId) -> Vec<ValueId> {
        self.allocation_points.get(&block_id).map_or(Vec::default(), |allocations| {
            allocations.iter().flat_map(|(_, constants)| constants).copied().collect()
        })
    }

    /// Collect all constants allocated in a given block at a specific location.
    pub(crate) fn allocated_at_location(
        &self,
        block_id: BasicBlockId,
        location: InstructionLocation,
    ) -> Option<&[ValueId]> {
        let allocations = self.allocation_points.get(&block_id)?;
        let constants = allocations.get(&location)?;
        Some(constants.as_ref())
    }

    /// Visit all constant variables in the function and record their locations.
    fn collect_constant_usage(&mut self, func: &Function) {
        let mut record_if_constant =
            |block_id: BasicBlockId, value_id: ValueId, location: InstructionLocation| {
                if is_numeric_constant(value_id, &func.dfg) {
                    self.constant_usage
                        .entry(value_id)
                        .or_default()
                        .entry(block_id)
                        .or_default()
                        .push(location);
                }
            };
        for block_id in func.reachable_blocks() {
            let block = &func.dfg[block_id];
            for &inst_id in block.instructions() {
                let variables = variables_used_in_instruction(&func.dfg[inst_id], &func.dfg);
                for variable in variables {
                    record_if_constant(
                        block_id,
                        variable,
                        InstructionLocation::Instruction(inst_id),
                    );
                }
            }
            if let Some(terminator_instruction) = block.terminator() {
                terminator_instruction.for_each_value(|value_id| {
                    if is_variable(value_id, &func.dfg) {
                        record_if_constant(block_id, value_id, InstructionLocation::Terminator);
                    }
                });
            }
        }
    }

    /// Based on the [Self::constant_usage] collected, find the common dominator of all the block where a constant is used
    /// and mark it as the allocation point for the constant.
    fn decide_allocation_points(&mut self, func: &Function) {
        for (constant_id, usage_in_blocks) in self.constant_usage.iter() {
            let block_ids: Vec<_> = usage_in_blocks.keys().copied().collect();

            let allocation_point = self.decide_allocation_point(*constant_id, &block_ids, func);

            // If the allocation point is one of the places where it's used, we take the first usage in the allocation point.
            // Otherwise, we allocate it at the terminator of the allocation point.
            let location = if let Some(locations_in_common_dominator) =
                usage_in_blocks.get(&allocation_point)
            {
                *locations_in_common_dominator
                    .first()
                    .expect("At least one location must have been found")
            } else {
                InstructionLocation::Terminator
            };

            self.allocation_points
                .entry(allocation_point)
                .or_default()
                .entry(location)
                .or_default()
                .push(*constant_id);
        }
    }

    /// Decide where to allocate a constant, based on the common dominator of the provided block vector.
    fn decide_allocation_point(
        &self,
        constant_id: ValueId,
        used_in_blocks: &[BasicBlockId],
        func: &Function,
    ) -> BasicBlockId {
        // Find the common dominator of all the blocks where the constant is used.
        let common_dominator = used_in_blocks
            .iter()
            .copied()
            .reduce(|a, b| self.dominator_tree.common_dominator(a, b))
            .unwrap_or(used_in_blocks[0]);

        // If the value only contains constants, it's safe to hoist outside of any loop.
        // Technically we know this is going to be true, because we only collected values which are `Value::NumericConstant`.
        if func.dfg.is_constant(constant_id) {
            self.exit_loops(common_dominator)
        } else {
            common_dominator
        }
    }

    /// Returns the nearest dominator that is outside of any loop.
    fn exit_loops(&self, block: BasicBlockId) -> BasicBlockId {
        let mut current_block = block;
        while self.blocks_within_loops.contains(&current_block) {
            current_block = self
                .dominator_tree
                .immediate_dominator(current_block)
                .expect("No dominator found when trying to allocate a constant outside of a loop");
        }
        current_block
    }

    /// Return the SSA [ValueId] of all constants (the same numeric constant might appear with multiple IDs).
    pub(crate) fn get_constants(&self) -> BTreeSet<ValueId> {
        self.constant_usage.keys().copied().collect()
    }
}

fn is_numeric_constant(id: ValueId, dfg: &DataFlowGraph) -> bool {
    matches!(&dfg[id], Value::NumericConstant { .. })
}
