//! This module analyzes the usage of constants in a given function and decides an allocation point for them.
//! The allocation point will be the common dominator of all the places where the constant is used.
//! By allocating in the common dominator, we can cache the constants for all subsequent uses.
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::ir::{
    basic_block::BasicBlockId,
    cfg::ControlFlowGraph,
    dfg::DataFlowGraph,
    dom::DominatorTree,
    function::Function,
    instruction::InstructionId,
    post_order::PostOrder,
    value::{Value, ValueId},
};

use super::variable_liveness::{collect_variables_of_value, variables_used_in_instruction};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum InstructionLocation {
    Instruction(InstructionId),
    Terminator,
}

pub(crate) struct ConstantAllocation {
    constant_usage: HashMap<ValueId, HashMap<BasicBlockId, Vec<InstructionLocation>>>,
    allocation_points: HashMap<BasicBlockId, HashMap<InstructionLocation, Vec<ValueId>>>,
    dominator_tree: DominatorTree,
    blocks_within_loops: HashSet<BasicBlockId>,
}

impl ConstantAllocation {
    pub(crate) fn from_function(func: &Function) -> Self {
        let cfg = ControlFlowGraph::with_function(func);
        let post_order = PostOrder::with_function(func);
        let mut dominator_tree = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);
        let blocks_within_loops = find_all_blocks_within_loops(func, &cfg, &mut dominator_tree);
        let mut instance = ConstantAllocation {
            constant_usage: HashMap::default(),
            allocation_points: HashMap::default(),
            dominator_tree,
            blocks_within_loops,
        };
        instance.collect_constant_usage(func);
        instance.decide_allocation_points(func);

        instance
    }

    pub(crate) fn allocated_in_block(&self, block_id: BasicBlockId) -> Vec<ValueId> {
        self.allocation_points.get(&block_id).map_or(Vec::default(), |allocations| {
            allocations.iter().flat_map(|(_, constants)| constants.iter()).copied().collect()
        })
    }

    pub(crate) fn allocated_at_location(
        &self,
        block_id: BasicBlockId,
        location: InstructionLocation,
    ) -> Vec<ValueId> {
        self.allocation_points.get(&block_id).map_or(Vec::default(), |allocations| {
            allocations.get(&location).map_or(Vec::default(), |constants| constants.clone())
        })
    }

    fn collect_constant_usage(&mut self, func: &Function) {
        let mut record_if_constant =
            |block_id: BasicBlockId, value_id: ValueId, location: InstructionLocation| {
                if is_constant_value(value_id, &func.dfg) {
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
                    if let Some(variable) = collect_variables_of_value(value_id, &func.dfg) {
                        record_if_constant(block_id, variable, InstructionLocation::Terminator);
                    }
                });
            }
        }
    }

    fn decide_allocation_points(&mut self, func: &Function) {
        for (constant_id, usage_in_blocks) in self.constant_usage.iter() {
            let block_ids: Vec<_> = usage_in_blocks.iter().map(|(block_id, _)| *block_id).collect();

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

    fn decide_allocation_point(
        &self,
        constant_id: ValueId,
        blocks_where_is_used: &[BasicBlockId],
        func: &Function,
    ) -> BasicBlockId {
        // Find the common dominator of all the blocks where the constant is used.
        let common_dominator = if blocks_where_is_used.len() == 1 {
            blocks_where_is_used[0]
        } else {
            let mut common_dominator = blocks_where_is_used[0];

            for block_id in blocks_where_is_used.iter().skip(1) {
                common_dominator =
                    self.dominator_tree.common_dominator(common_dominator, *block_id);
            }

            common_dominator
        };
        // If the value only contains constants, it's safe to hoist outside of any loop
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
}

pub(crate) fn is_constant_value(id: ValueId, dfg: &DataFlowGraph) -> bool {
    matches!(&dfg[dfg.resolve(id)], Value::NumericConstant { .. })
}

/// For a given function, finds all the blocks that are within loops
fn find_all_blocks_within_loops(
    func: &Function,
    cfg: &ControlFlowGraph,
    dominator_tree: &mut DominatorTree,
) -> HashSet<BasicBlockId> {
    let mut blocks_in_loops = HashSet::default();
    for block_id in func.reachable_blocks() {
        let block = &func.dfg[block_id];
        let successors = block.successors();
        for successor_id in successors {
            if dominator_tree.dominates(successor_id, block_id) {
                blocks_in_loops.extend(find_blocks_in_loop(successor_id, block_id, cfg));
            }
        }
    }

    blocks_in_loops
}

/// Return each block that is in a loop starting in the given header block.
/// Expects back_edge_start -> header to be the back edge of the loop.
fn find_blocks_in_loop(
    header: BasicBlockId,
    back_edge_start: BasicBlockId,
    cfg: &ControlFlowGraph,
) -> HashSet<BasicBlockId> {
    let mut blocks = HashSet::default();
    blocks.insert(header);

    let mut insert = |block, stack: &mut Vec<BasicBlockId>| {
        if !blocks.contains(&block) {
            blocks.insert(block);
            stack.push(block);
        }
    };

    // Starting from the back edge of the loop, each predecessor of this block until
    // the header is within the loop.
    let mut stack = vec![];
    insert(back_edge_start, &mut stack);

    while let Some(block) = stack.pop() {
        for predecessor in cfg.predecessors(block) {
            insert(predecessor, &mut stack);
        }
    }

    blocks
}
