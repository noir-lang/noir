//! This module analyzes the usage of constants in a given function and decides an allocation point for them.
//! The allocation point will be the common dominator of all the places where the constant is used.
//! By allocating in the common dominator, we can cache the constants for all subsequent uses.
use fxhash::FxHashMap as HashMap;

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
}

impl ConstantAllocation {
    pub(crate) fn from_function(func: &Function) -> Self {
        let cfg = ControlFlowGraph::with_function(func);
        let post_order = PostOrder::with_function(func);
        let dominator_tree = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);
        let mut instance = ConstantAllocation {
            constant_usage: HashMap::default(),
            allocation_points: HashMap::default(),
            dominator_tree,
        };
        instance.collect_constant_usage(func);
        instance.decide_allocation_points();

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
                    let variables = collect_variables_of_value(value_id, &func.dfg);
                    for variable in variables {
                        record_if_constant(block_id, variable, InstructionLocation::Terminator);
                    }
                });
            }
        }
    }

    fn decide_allocation_points(&mut self) {
        for (constant_id, usage_in_blocks) in self.constant_usage.iter() {
            let block_ids: Vec<_> = usage_in_blocks.iter().map(|(block_id, _)| *block_id).collect();

            let common_dominator = self.common_dominator(&block_ids);

            // If the common dominator is one of the places where it's used, we take the first usage in the common dominator.
            // Otherwise, we allocate it at the terminator of the common dominator.
            let location = if let Some(locations_in_common_dominator) =
                usage_in_blocks.get(&common_dominator)
            {
                *locations_in_common_dominator
                    .first()
                    .expect("At least one location must have been found")
            } else {
                InstructionLocation::Terminator
            };

            self.allocation_points
                .entry(common_dominator)
                .or_default()
                .entry(location)
                .or_default()
                .push(*constant_id);
        }
    }

    fn common_dominator(&self, block_ids: &[BasicBlockId]) -> BasicBlockId {
        if block_ids.len() == 1 {
            return block_ids[0];
        }

        let mut common_dominator = block_ids[0];

        for block_id in block_ids.iter().skip(1) {
            common_dominator = self.dominator_tree.common_dominator(common_dominator, *block_id);
        }

        common_dominator
    }
}

pub(crate) fn is_constant_value(id: ValueId, dfg: &DataFlowGraph) -> bool {
    matches!(&dfg[dfg.resolve(id)], Value::NumericConstant { .. } | Value::Array { .. })
}
