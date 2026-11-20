use std::collections::BTreeSet;

use super::{
    basic_block::{BasicBlock, BasicBlockId},
    function::Function,
    map::DenseSideTable,
};
use smallvec::SmallVec;

/// A set of blocks, sorted by id so that iterating it is deterministic.
///
/// Most blocks have at most two successors and two predecessors, so the set is stored inline.
#[derive(Clone, Default)]
struct BlockSet(SmallVec<[BasicBlockId; 2]>);

impl BlockSet {
    /// Inserts `block`, returning `false` if it was already present.
    ///
    /// Edges are usually added in increasing block order, which makes this an append.
    fn insert(&mut self, block: BasicBlockId) -> bool {
        if self.0.last().is_none_or(|last| *last < block) {
            self.0.push(block);
            return true;
        }
        match self.0.binary_search(&block) {
            Ok(_) => false,
            Err(index) => {
                self.0.insert(index, block);
                true
            }
        }
    }

    fn remove(&mut self, block: BasicBlockId) {
        if let Ok(index) = self.0.binary_search(&block) {
            self.0.remove(index);
        }
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn iter(&self) -> std::iter::Copied<std::slice::Iter<'_, BasicBlockId>> {
        self.0.iter().copied()
    }
}

/// A container for the successors and predecessors of some Block.
#[derive(Clone, Default)]
struct CfgNode {
    /// Blocks containing jumps that target this block, in increasing id order.
    predecessors: BlockSet,

    /// Blocks this block jumps to, in increasing id order.
    successors: BlockSet,
}

#[derive(Clone, Default)]
/// The Control Flow Graph (CFG) maintains a mapping of blocks to their predecessors
/// and successors where predecessors are basic blocks and successors are
/// basic blocks.
pub(crate) struct ControlFlowGraph {
    /// The node of each block in the CFG, indexed by block id. A block is in the CFG when its
    /// slot is `Some`: the entry block, and every block at either end of an edge.
    data: DenseSideTable<BasicBlock, Option<CfgNode>>,
    /// Flag stating whether this CFG has been reversed.
    /// In a reversed CFG, successors become predecessors.
    reversed: bool,
}

impl ControlFlowGraph {
    /// Allocate and compute the control flow graph for `func`.
    pub(crate) fn with_function(func: &Function) -> Self {
        // It is expected to be safe to query the control flow graph for any reachable block,
        // therefore we must ensure that a node exists for the entry block, regardless of whether
        // it later comes to describe any edges after calling compute.
        let mut cfg = ControlFlowGraph::default();
        cfg.node_mut(func.entry_block());
        cfg.compute(func);
        cfg
    }

    /// Compute all of the edges between each reachable block in the function
    fn compute(&mut self, func: &Function) {
        for basic_block_id in func.reachable_blocks() {
            let basic_block = &func.dfg[basic_block_id];
            self.compute_block(basic_block_id, basic_block);
        }
    }

    /// Compute all of the edges for the current block given
    fn compute_block(&mut self, basic_block_id: BasicBlockId, basic_block: &BasicBlock) {
        for dest in basic_block.successors() {
            self.add_edge(basic_block_id, dest);
        }
    }

    /// The node of `block`, if `block` is in the CFG.
    fn node(&self, block: BasicBlockId) -> Option<&CfgNode> {
        self.data.get(block).and_then(Option::as_ref)
    }

    /// The node of `block`, adding an empty one if `block` is not in the CFG yet.
    fn node_mut(&mut self, block: BasicBlockId) -> &mut CfgNode {
        if self.data.get(block).is_none() {
            // Grow the table to cover `block`.
            self.data.set(block, None);
        }
        self.data[block].get_or_insert_with(CfgNode::default)
    }

    /// Iterates over the blocks in the CFG, in increasing id order.
    fn blocks(&self) -> impl Iterator<Item = BasicBlockId> + '_ {
        self.data.iter().filter_map(|(block, node)| node.as_ref().map(|_| block))
    }

    /// Clears out a given block's successors. This also removes the given block from
    /// being a predecessor of any of its previous successors.
    pub(crate) fn invalidate_block_successors(&mut self, basic_block_id: BasicBlockId) {
        let node = self
            .data
            .get_mut(basic_block_id)
            .and_then(Option::as_mut)
            .expect("ICE: Attempted to invalidate cfg node successors for non-existent node.");

        let old_successors = std::mem::take(&mut node.successors);

        for successor_id in old_successors.iter() {
            self.data
                .get_mut(successor_id)
                .and_then(Option::as_mut)
                .expect("ICE: Cfg node successor doesn't exist.")
                .predecessors
                .remove(basic_block_id);
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

    /// Add a directed edge making `from` a predecessor of `to`.
    fn add_edge(&mut self, from: BasicBlockId, to: BasicBlockId) {
        let reversed = self.reversed;
        let predecessor_node = self.node_mut(from);
        if !reversed {
            assert!(
                predecessor_node.successors.len() < 2,
                "ICE: A cfg node cannot have more than two successors"
            );
        }
        predecessor_node.successors.insert(to);
        let successor_node = self.node_mut(to);
        successor_node.predecessors.insert(from);
    }

    /// Get an iterator over the CFG predecessors to `basic_block_id`.
    pub(crate) fn predecessors(
        &self,
        basic_block_id: BasicBlockId,
    ) -> impl ExactSizeIterator<Item = BasicBlockId> + '_ {
        self.node(basic_block_id)
            .expect("ICE: Attempted to iterate predecessors of block not found within cfg.")
            .predecessors
            .iter()
    }

    /// The blocks of the natural loop with the given `header` and back-edge source
    /// `back_edge_start`: the header together with every block that reaches `back_edge_start`
    /// by walking predecessors backwards, stopping at the header.
    pub(crate) fn find_blocks_in_loop(
        &self,
        header: BasicBlockId,
        back_edge_start: BasicBlockId,
    ) -> BTreeSet<BasicBlockId> {
        let mut blocks = BTreeSet::new();
        blocks.insert(header);

        let mut stack = vec![back_edge_start];
        while let Some(block) = stack.pop() {
            // The header is already inserted, so reaching it returns `false` and stops the walk
            // past it; this also terminates a single-block (self-edge) loop.
            if blocks.insert(block) {
                stack.extend(self.predecessors(block));
            }
        }
        blocks
    }

    /// Get an iterator over the CFG successors to `basic_block_id`.
    pub(crate) fn successors(
        &self,
        basic_block_id: BasicBlockId,
    ) -> impl ExactSizeIterator<Item = BasicBlockId> + DoubleEndedIterator + '_ {
        self.node(basic_block_id)
            .expect("ICE: Attempted to iterate successors of block not found within cfg.")
            .successors
            .iter()
    }

    /// Reverse the control flow graph
    pub(crate) fn reverse(&self) -> Self {
        let mut reversed_cfg = ControlFlowGraph { reversed: true, ..Default::default() };

        for block_id in self.blocks() {
            // For each block, reverse the edges
            // In the reversed CFG, successors becomes predecessors
            for successor in self.successors(block_id) {
                reversed_cfg.add_edge(successor, block_id);
            }
        }

        reversed_cfg
    }

    /// Returns the entry blocks for a CFG. This is all nodes without any predecessors.
    pub(crate) fn compute_entry_blocks(&self) -> Vec<BasicBlockId> {
        self.blocks().filter(|&block| self.predecessors(block).len() == 0).collect()
    }

    /// Computes the reverse graph of the extended CFG.
    /// The extended CFG is the CFG with an additional unique exit node (if there is none)
    /// such that there is a path from every block to the exit node.
    /// Ex: below the forward CFG has one exit node: b2
    /// However, there is no path from b5 to b2
    /// ```text
    /// forward          reverse
    ///  -------          -------
    ///   b0*              b0
    ///   |                ^
    ///   v                |
    ///   b1               b1
    ///  /  \             ^  ^
    /// v    v           /    \
    /// b3   b4          b3   b4
    /// |    |           ^    ^
    /// v    v           |    |
    /// b2   b5 <-|      b2*  b5 <-|
    ///       \___|            \___|
    /// ```
    ///
    /// The extended CFG is the forward CFG with a new 'exit' node:
    /// ```text
    ///  extended         extended reverse
    ///  -------          -------
    ///   b0*              b0
    ///   |                ^
    ///   v                |
    ///   b1               b1
    ///  /  \             ^  ^
    /// v    v           /    \
    /// b3   b4          b3   b4
    /// |    |           ^    ^
    /// v    v           |    |
    /// b2   b5 <-|      b2   b5 <-|
    /// \    /\___|      ^    ^\___|
    ///  v  v            \    /
    ///  exit             exit*
    /// ```
    pub(crate) fn extended_reverse(func: &mut Function) -> Self {
        let mut cfg = Self::with_function(func);
        // Exit blocks are the ones having no successor
        let exit_nodes: Vec<BasicBlockId> =
            cfg.blocks().filter(|&block| cfg.successors(block).len() == 0).collect();
        // Traverse the reverse CFG from the exit blocks
        let reverse = cfg.reverse();
        let post_order = crate::ssa::ir::post_order::PostOrder::with_cfg(&reverse);
        // Extract blocks that are not reachable from the exit blocks
        let mut rpo_traversal = DenseSideTable::<BasicBlock, bool>::default();
        for block in post_order.into_vec() {
            rpo_traversal.mark(block);
        }
        let dead_blocks: Vec<BasicBlockId> =
            cfg.blocks().filter(|block| !rpo_traversal.contains(*block)).collect();

        // If some blocks, that we call 'dead' blocks, are not in the post-order traversal of the reverse CFG,
        // or if there are multiple exit nodes, then the reverse CFG is not a CFG because
        // it does not have a single entry node and so we will not be able to apply the dominance frontier algorithm.
        // In that case, we extend the CFG with a new 'exit' node and connect the exit blocks and the 'dead' blocks to it.
        if exit_nodes.len() > 1 || !dead_blocks.is_empty() {
            // Create a fake 'exit' block
            let exit = func.dfg.make_block();
            cfg.node_mut(exit);
            // Connect the exit nodes to it
            for e in exit_nodes {
                cfg.add_edge(e, exit);
            }
            // Connect the 'dead' blocks to it
            for block in dead_blocks {
                cfg.add_edge(block, exit);
            }
        }
        // We can now reverse the extended CFG
        cfg.reverse()
    }
}

#[cfg(test)]
mod tests {
    use crate::ssa::ir::{
        basic_block::BasicBlockId, instruction::TerminatorInstruction, map::Id, types::Type,
    };
    use noirc_errors::call_stack::CallStackId;

    use super::{super::function::Function, BlockSet, ControlFlowGraph};

    #[test]
    fn block_set_stays_sorted_and_deduplicated() {
        let b = |id| BasicBlockId::test_new(id);
        let mut set = BlockSet::default();
        assert!(set.insert(b(5)));
        assert!(set.insert(b(1)));
        assert!(set.insert(b(9)));
        assert!(set.insert(b(3)));
        assert!(!set.insert(b(5)));
        assert!(!set.insert(b(9)));
        assert_eq!(set.iter().collect::<Vec<_>>(), vec![b(1), b(3), b(5), b(9)]);

        set.remove(b(3));
        set.remove(b(4));
        assert_eq!(set.iter().collect::<Vec<_>>(), vec![b(1), b(5), b(9)]);
    }

    #[test]
    fn empty() {
        let func_id = Id::test_new(0);
        let mut func = Function::new("func".into(), func_id);
        let block_id = func.entry_block();
        func.dfg[block_id].set_terminator(TerminatorInstruction::Return {
            return_values: vec![],
            call_stack: CallStackId::root(),
        });

        ControlFlowGraph::with_function(&func);
    }

    fn build_test_function() -> (Function, BasicBlockId, BasicBlockId, BasicBlockId) {
        // Build function of form
        // fn func {
        //   block0(cond: u1):
        //     jmpif cond, then: block2, else: block1
        //   block1():
        //     jmpif cond, then: block1, else: block2
        //   block2():
        //     return ()
        // }
        let func_id = Id::test_new(0);
        let mut func = Function::new("func".into(), func_id);
        let block0_id = func.entry_block();
        let cond = func.dfg.add_block_parameter(block0_id, Type::unsigned(1));
        let block1_id = func.dfg.make_block();
        let block2_id = func.dfg.make_block();

        func.dfg[block0_id].set_terminator(TerminatorInstruction::JmpIf {
            condition: cond,
            then_destination: block2_id,
            then_arguments: Vec::new(),
            else_destination: block1_id,
            else_arguments: Vec::new(),
            call_stack: CallStackId::root(),
        });
        func.dfg[block1_id].set_terminator(TerminatorInstruction::JmpIf {
            condition: cond,
            then_destination: block1_id,
            then_arguments: Vec::new(),
            else_destination: block2_id,
            else_arguments: Vec::new(),
            call_stack: CallStackId::root(),
        });
        func.dfg[block2_id].set_terminator(TerminatorInstruction::Return {
            return_values: vec![],
            call_stack: CallStackId::root(),
        });

        (func, block0_id, block1_id, block2_id)
    }

    fn modify_test_function(
        func: &mut Function,
        block0_id: BasicBlockId,
        block1_id: BasicBlockId,
        block2_id: BasicBlockId,
    ) -> BasicBlockId {
        // Modify function to form:
        // fn func {
        //   block0(cond: u1):
        //     jmpif cond, then: block1, else: ret_block
        //   block1():
        //     jmpif cond, then: block1, else: block2
        //   block2():
        //     jmp ret_block()
        //   ret_block():
        //     return ()
        // }
        let ret_block_id = func.dfg.make_block();
        func.dfg[ret_block_id].set_terminator(TerminatorInstruction::Return {
            return_values: vec![],
            call_stack: CallStackId::root(),
        });
        func.dfg[block2_id].set_terminator(TerminatorInstruction::Jmp {
            destination: ret_block_id,
            arguments: vec![],
            call_stack: CallStackId::root(),
        });
        let cond = func.dfg[block0_id].parameters()[0];
        func.dfg[block0_id].set_terminator(TerminatorInstruction::JmpIf {
            condition: cond,
            then_destination: block1_id,
            then_arguments: Vec::new(),
            else_destination: ret_block_id,
            else_arguments: Vec::new(),
            call_stack: CallStackId::root(),
        });
        ret_block_id
    }

    #[test]
    fn jumps() {
        let (mut func, block0_id, block1_id, block2_id) = build_test_function();

        let mut cfg = ControlFlowGraph::with_function(&func);

        #[allow(clippy::needless_collect)]
        {
            let block0_predecessors: Vec<_> = cfg.predecessors(block0_id).collect();
            let block1_predecessors: Vec<_> = cfg.predecessors(block1_id).collect();
            let block2_predecessors: Vec<_> = cfg.predecessors(block2_id).collect();

            let block0_successors: Vec<_> = cfg.successors(block0_id).collect();
            let block1_successors: Vec<_> = cfg.successors(block1_id).collect();
            let block2_successors: Vec<_> = cfg.successors(block2_id).collect();

            assert_eq!(block0_predecessors.len(), 0);
            assert_eq!(block1_predecessors.len(), 2);
            assert_eq!(block2_predecessors.len(), 2);

            assert!(block1_predecessors.contains(&block0_id));
            assert!(block1_predecessors.contains(&block1_id));
            assert!(block2_predecessors.contains(&block0_id));
            assert!(block2_predecessors.contains(&block1_id));

            assert_eq!(block0_successors.len(), 2);
            assert_eq!(block1_successors.len(), 2);
            assert_eq!(block2_successors.len(), 0);

            assert!(block0_successors.contains(&block1_id));
            assert!(block0_successors.contains(&block2_id));
            assert!(block1_successors.contains(&block1_id));
            assert!(block1_successors.contains(&block2_id));
        }

        let ret_block_id = modify_test_function(&mut func, block0_id, block1_id, block2_id);

        // Recompute new and changed blocks
        cfg.recompute_block(&func, block0_id);
        cfg.recompute_block(&func, block2_id);
        cfg.recompute_block(&func, ret_block_id);

        #[allow(clippy::needless_collect)]
        {
            let block0_predecessors: Vec<_> = cfg.predecessors(block0_id).collect();
            let block1_predecessors: Vec<_> = cfg.predecessors(block1_id).collect();
            let block2_predecessors: Vec<_> = cfg.predecessors(block2_id).collect();

            let block0_successors: Vec<_> = cfg.successors(block0_id).collect();
            let block1_successors: Vec<_> = cfg.successors(block1_id).collect();
            let block2_successors: Vec<_> = cfg.successors(block2_id).collect();

            assert_eq!(block0_predecessors.len(), 0);
            assert_eq!(block1_predecessors.len(), 2);
            assert_eq!(block2_predecessors.len(), 1);

            assert!(block1_predecessors.contains(&block0_id));
            assert!(block1_predecessors.contains(&block1_id));
            assert!(!block2_predecessors.contains(&block0_id));
            assert!(block2_predecessors.contains(&block1_id));

            assert_eq!(block0_successors.len(), 2);
            assert_eq!(block1_successors.len(), 2);
            assert_eq!(block2_successors.len(), 1);

            assert!(block0_successors.contains(&block1_id));
            assert!(block0_successors.contains(&ret_block_id));
            assert!(block1_successors.contains(&block1_id));
            assert!(block1_successors.contains(&block2_id));
            assert!(block2_successors.contains(&ret_block_id));
        }
    }

    #[test]
    fn reversed_cfg_jumps() {
        let (mut func, block0_id, block1_id, block2_id) = build_test_function();

        let mut cfg = ControlFlowGraph::with_function(&func);
        let reversed_cfg = cfg.reverse();

        #[allow(clippy::needless_collect)]
        {
            let block0_predecessors: Vec<_> = reversed_cfg.predecessors(block0_id).collect();
            let block1_predecessors: Vec<_> = reversed_cfg.predecessors(block1_id).collect();
            let block2_predecessors: Vec<_> = reversed_cfg.predecessors(block2_id).collect();

            let block0_successors: Vec<_> = reversed_cfg.successors(block0_id).collect();
            let block1_successors: Vec<_> = reversed_cfg.successors(block1_id).collect();
            let block2_successors: Vec<_> = reversed_cfg.successors(block2_id).collect();

            assert_eq!(block0_predecessors.len(), 2);
            assert_eq!(block1_predecessors.len(), 2);
            assert_eq!(block2_predecessors.len(), 0);

            assert!(block0_predecessors.contains(&block1_id));
            assert!(block0_predecessors.contains(&block2_id));
            assert!(block1_predecessors.contains(&block1_id));
            assert!(block1_predecessors.contains(&block2_id));

            assert_eq!(block0_successors.len(), 0);
            assert_eq!(block1_successors.len(), 2);
            assert_eq!(block2_successors.len(), 2);

            assert!(block1_successors.contains(&block0_id));
            assert!(block1_successors.contains(&block1_id));
            assert!(block2_successors.contains(&block0_id));
            assert!(block2_successors.contains(&block1_id));
        }

        let ret_block_id = modify_test_function(&mut func, block0_id, block1_id, block2_id);

        // Recompute new and changed blocks
        cfg.recompute_block(&func, block0_id);
        cfg.recompute_block(&func, block2_id);
        cfg.recompute_block(&func, ret_block_id);

        let reversed_cfg = cfg.reverse();

        #[allow(clippy::needless_collect)]
        {
            let block0_predecessors: Vec<_> = reversed_cfg.predecessors(block0_id).collect();
            let block1_predecessors: Vec<_> = reversed_cfg.predecessors(block1_id).collect();
            let block2_predecessors: Vec<_> = reversed_cfg.predecessors(block2_id).collect();
            let ret_block_predecessors: Vec<_> = reversed_cfg.predecessors(ret_block_id).collect();

            let block0_successors: Vec<_> = reversed_cfg.successors(block0_id).collect();
            let block1_successors: Vec<_> = reversed_cfg.successors(block1_id).collect();
            let block2_successors: Vec<_> = reversed_cfg.successors(block2_id).collect();
            let ret_block_successors: Vec<_> = reversed_cfg.successors(ret_block_id).collect();

            assert_eq!(block0_predecessors.len(), 2);
            assert_eq!(block1_predecessors.len(), 2);
            assert_eq!(block2_predecessors.len(), 1);
            assert_eq!(ret_block_predecessors.len(), 0);

            assert!(block0_predecessors.contains(&block1_id));
            assert!(block0_predecessors.contains(&ret_block_id));
            assert!(block1_predecessors.contains(&block1_id));
            assert!(block1_predecessors.contains(&block2_id));
            assert!(!block2_predecessors.contains(&block0_id));
            assert!(block2_predecessors.contains(&ret_block_id));

            assert_eq!(block0_successors.len(), 0);
            assert_eq!(block1_successors.len(), 2);
            assert_eq!(block2_successors.len(), 1);
            assert_eq!(ret_block_successors.len(), 2);

            assert!(block1_successors.contains(&block0_id));
            assert!(block1_successors.contains(&block1_id));
            assert!(block2_successors.contains(&block1_id));
            assert!(ret_block_successors.contains(&block0_id));
            assert!(ret_block_successors.contains(&block2_id));
        }
    }
}
