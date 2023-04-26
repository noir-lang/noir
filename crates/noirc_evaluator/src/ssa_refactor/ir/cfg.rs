use std::collections::{HashMap, HashSet};

use super::{
    basic_block::{BasicBlock, BasicBlockId},
    basic_block_views,
    function::Function,
};

/// A container for the successors and predecessors of some Block.
#[derive(Clone, Default)]
struct CfgNode {
    /// Set of blocks that containing jumps that target this block.
    /// The predecessor set has no meaningful order.
    pub(crate) predecessors: HashSet<BasicBlockId>,

    /// Set of blocks that are the targets of jumps in this block.
    /// The successors set has no meaningful order.
    pub(crate) successors: HashSet<BasicBlockId>,
}

/// The Control Flow Graph maintains a mapping of blocks to their predecessors
/// and successors where predecessors are basic blocks and successors are
/// basic blocks.
pub(crate) struct ControlFlowGraph {
    data: HashMap<BasicBlockId, CfgNode>,
}

impl ControlFlowGraph {
    /// Allocate and compute the control flow graph for `func`.
    pub(crate) fn with_function(func: &Function) -> Self {
        let mut cfg = ControlFlowGraph { data: HashMap::new() };
        cfg.compute(func);
        cfg
    }

    fn compute(&mut self, func: &Function) {
        for (basic_block_id, basic_block) in func.dfg.basic_blocks_iter() {
            self.compute_block(basic_block_id, basic_block);
        }
    }

    fn compute_block(&mut self, basic_block_id: BasicBlockId, basic_block: &BasicBlock) {
        for dest in basic_block_views::successors_iter(basic_block) {
            self.add_edge(basic_block_id, dest);
        }
    }

    fn invalidate_block_successors(&mut self, basic_block_id: BasicBlockId) {
        let node = self
            .data
            .get_mut(&basic_block_id)
            .expect("ICE: Attempted to invalidate cfg node successors for non-existent node.");
        let old_successors = node.successors.clone();
        node.successors.clear();
        for successor_id in old_successors {
            self.data
                .get_mut(&successor_id)
                .expect("ICE: Cfg node successor doesn't exist.")
                .predecessors
                .remove(&basic_block_id);
        }
    }

    /// Recompute the control flow graph of `block`.
    ///
    /// This is for use after modifying instructions within a specific block. It recomputes all edges
    /// from `basic_block_id` while leaving edges to `basic_block_id` intact.
    pub(crate) fn recompute_block(&mut self, func: &Function, basic_block_id: BasicBlockId) {
        self.invalidate_block_successors(basic_block_id);
        let basic_block = &func.dfg[basic_block_id];
        self.compute_block(basic_block_id, basic_block);
    }

    fn add_edge(&mut self, from: BasicBlockId, to: BasicBlockId) {
        let predecessor_node = self.data.entry(from).or_default();
        assert!(
            predecessor_node.successors.len() < 2,
            "ICE: A cfg node cannot have more than two successors"
        );
        predecessor_node.successors.insert(to);
        let successor_node = self.data.entry(to).or_default();
        assert!(
            successor_node.predecessors.len() < 2,
            "ICE: A cfg node cannot have more than two predecessors"
        );
        successor_node.predecessors.insert(from);
    }

    /// Get an iterator over the CFG predecessors to `basic_block_id`.
    pub(crate) fn pred_iter(
        &self,
        basic_block_id: BasicBlockId,
    ) -> impl ExactSizeIterator<Item = BasicBlockId> + '_ {
        self.data
            .get(&basic_block_id)
            .expect("ICE: Attempted to iterate predecessors of block not found within cfg.")
            .predecessors
            .iter()
            .copied()
    }

    /// Get an iterator over the CFG successors to `basic_block_id`.
    pub(crate) fn succ_iter(
        &self,
        basic_block_id: BasicBlockId,
    ) -> impl ExactSizeIterator<Item = BasicBlockId> + '_ {
        self.data
            .get(&basic_block_id)
            .expect("ICE: Attempted to iterate successors of block not found within cfg.")
            .successors
            .iter()
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use crate::ssa_refactor::ir::{instruction::TerminatorInstruction, types::Type};

    use super::{super::function::Function, ControlFlowGraph};

    #[test]
    fn empty() {
        let mut func = Function::new("func".into());
        let block_id = func.entry_block();
        func.dfg[block_id].set_terminator(TerminatorInstruction::Return { return_values: vec![] });

        ControlFlowGraph::with_function(&func);
    }

    #[test]
    fn jumps() {
        // Build function of form
        // fn func {
        // block0(cond: u1):
        //     jmpif cond(), then: block2, else: block1
        // block1():
        //     jmpif cond(), then: block1, else: block2
        // block2():
        //     return
        // }
        let mut func = Function::new("func".into());
        let block0_id = func.entry_block();
        let cond = func.dfg.add_block_parameter(block0_id, Type::unsigned(1));
        let block1_id = func.dfg.new_block();
        let block2_id = func.dfg.new_block();

        func.dfg[block0_id].set_terminator(TerminatorInstruction::JmpIf {
            condition: cond,
            then_destination: block2_id,
            else_destination: block1_id,
            arguments: vec![],
        });
        func.dfg[block1_id].set_terminator(TerminatorInstruction::JmpIf {
            condition: cond,
            then_destination: block1_id,
            else_destination: block2_id,
            arguments: vec![],
        });
        func.dfg[block2_id].set_terminator(TerminatorInstruction::Return { return_values: vec![] });

        let mut cfg = ControlFlowGraph::with_function(&func);

        {
            let block0_predecessors = cfg.pred_iter(block0_id).collect::<Vec<_>>();
            let block1_predecessors = cfg.pred_iter(block1_id).collect::<Vec<_>>();
            let block2_predecessors = cfg.pred_iter(block2_id).collect::<Vec<_>>();

            let block0_successors = cfg.succ_iter(block0_id).collect::<Vec<_>>();
            let block1_successors = cfg.succ_iter(block1_id).collect::<Vec<_>>();
            let block2_successors = cfg.succ_iter(block2_id).collect::<Vec<_>>();

            assert_eq!(block0_predecessors.len(), 0);
            assert_eq!(block1_predecessors.len(), 2);
            assert_eq!(block2_predecessors.len(), 2);

            assert_eq!(block1_predecessors.contains(&block0_id), true);
            assert_eq!(block1_predecessors.contains(&block1_id), true);
            assert_eq!(block2_predecessors.contains(&block0_id), true);
            assert_eq!(block2_predecessors.contains(&block1_id), true);

            assert_eq!(block0_successors.len(), 2);
            assert_eq!(block1_successors.len(), 2);
            assert_eq!(block2_successors.len(), 0);

            assert_eq!(block0_successors.contains(&block1_id), true);
            assert_eq!(block0_successors.contains(&block2_id), true);
            assert_eq!(block1_successors.contains(&block1_id), true);
            assert_eq!(block1_successors.contains(&block2_id), true);
        }

        // Modify function to form:
        // fn func {
        // block0(cond: u1):
        //     jmpif cond(), then: block1, else: ret_block
        // block1():
        //     jmpif cond(), then: block1, else: block2
        // block2():
        //     jmp ret_block
        // ret_block():
        //     return
        // }
        let ret_block_id = func.dfg.new_block();
        func.dfg[ret_block_id]
            .set_terminator(TerminatorInstruction::Return { return_values: vec![] });
        func.dfg[block2_id].set_terminator(TerminatorInstruction::Jmp {
            destination: ret_block_id,
            arguments: vec![],
        });
        func.dfg[block0_id].set_terminator(TerminatorInstruction::JmpIf {
            condition: cond,
            then_destination: block1_id,
            else_destination: ret_block_id,
            arguments: vec![],
        });

        // Recompute new and changed blocks
        cfg.recompute_block(&mut func, block0_id);
        cfg.recompute_block(&mut func, block2_id);
        cfg.recompute_block(&mut func, ret_block_id);

        {
            let block0_predecessors = cfg.pred_iter(block0_id).collect::<Vec<_>>();
            let block1_predecessors = cfg.pred_iter(block1_id).collect::<Vec<_>>();
            let block2_predecessors = cfg.pred_iter(block2_id).collect::<Vec<_>>();

            let block0_successors = cfg.succ_iter(block0_id).collect::<Vec<_>>();
            let block1_successors = cfg.succ_iter(block1_id).collect::<Vec<_>>();
            let block2_successors = cfg.succ_iter(block2_id).collect::<Vec<_>>();

            assert_eq!(block0_predecessors.len(), 0);
            assert_eq!(block1_predecessors.len(), 2);
            assert_eq!(block2_predecessors.len(), 1);

            assert_eq!(block1_predecessors.contains(&block0_id), true);
            assert_eq!(block1_predecessors.contains(&block1_id), true);
            assert_eq!(block2_predecessors.contains(&block0_id), false);
            assert_eq!(block2_predecessors.contains(&block1_id), true);

            assert_eq!(block0_successors.len(), 2);
            assert_eq!(block1_successors.len(), 2);
            assert_eq!(block2_successors.len(), 1);

            assert_eq!(block0_successors.contains(&block1_id), true);
            assert_eq!(block0_successors.contains(&ret_block_id), true);
            assert_eq!(block1_successors.contains(&block1_id), true);
            assert_eq!(block1_successors.contains(&block2_id), true);
            assert_eq!(block2_successors.contains(&ret_block_id), true);
        }
    }
}
