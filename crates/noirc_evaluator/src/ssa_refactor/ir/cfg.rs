use std::collections::{hash_set, HashMap, HashSet};

use super::{
    basic_block::{BasicBlock, BasicBlockId},
    basic_block_visitors,
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
        for (basic_block_id, basic_block) in func.basic_blocks_iter() {
            self.compute_block(basic_block_id, basic_block);
        }
    }

    fn compute_block(&mut self, basic_block_id: BasicBlockId, basic_block: &BasicBlock) {
        basic_block_visitors::visit_block_succs(basic_block, |dest| {
            self.add_edge(basic_block_id, dest);
        });
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
        let basic_block = &func[basic_block_id];
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

    /// Get an iterator over the CFG predecessors to `block`.
    pub(crate) fn pred_iter(&self, basic_block_id: BasicBlockId) -> PredIter {
        self.data
            .get(&basic_block_id)
            .expect("ICE: Attempted to iterate predecessors of block not found within cfg.")
            .predecessors
            .iter()
    }

    /// Get an iterator over the CFG successors to `block`.
    pub(crate) fn succ_iter(&self, basic_block_id: BasicBlockId) -> SuccIter {
        self.data
            .get(&basic_block_id)
            .expect("ICE: Attempted to iterate successors of block not found within cfg.")
            .successors
            .iter()
    }
}

/// An iterator over block predecessors. The iterator type is `BasicBlockId`.
pub(crate) type PredIter<'a> = hash_set::Iter<'a, BasicBlockId>;

/// An iterator over block successors. The iterator type is `BasicBlockId`.
pub(crate) type SuccIter<'a> = hash_set::Iter<'a, BasicBlockId>;
