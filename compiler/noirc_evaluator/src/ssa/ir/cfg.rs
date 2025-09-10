use std::collections::BTreeSet;

use super::{
    basic_block::{BasicBlock, BasicBlockId},
    function::Function,
    post_order::PostOrder,
};
use rustc_hash::FxHashMap as HashMap;
use std::collections::HashSet;

/// A container for the successors and predecessors of some Block.
#[derive(Clone, Default, Debug)]
struct CfgNode {
    /// Set of blocks that containing jumps that target this block.
    /// The predecessor set has no meaningful order.
    pub(crate) predecessors: BTreeSet<BasicBlockId>,

    /// Set of blocks that are the targets of jumps in this block.
    /// The successors set has no meaningful order.
    pub(crate) successors: BTreeSet<BasicBlockId>,
}

#[derive(Clone, Default, Debug)]
/// The Control Flow Graph (CFG) maintains a mapping of blocks to their predecessors
/// and successors where predecessors are basic blocks and successors are
/// basic blocks.
pub(crate) struct ControlFlowGraph {
    data: HashMap<BasicBlockId, CfgNode>,
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
        let entry_block = func.entry_block();
        let empty_node = CfgNode { predecessors: BTreeSet::new(), successors: BTreeSet::new() };
        let mut data = HashMap::default();
        data.insert(entry_block, empty_node);

        let mut cfg = ControlFlowGraph { data, reversed: false };
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

    /// Clears out a given block's successors. This also removes the given block from
    /// being a predecessor of any of its previous successors.
    pub(crate) fn invalidate_block_successors(&mut self, basic_block_id: BasicBlockId) {
        let node = self
            .data
            .get_mut(&basic_block_id)
            .expect("ICE: Attempted to invalidate cfg node successors for non-existent node.");

        let old_successors = std::mem::take(&mut node.successors);

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

    /// Add a directed edge making `from` a predecessor of `to`.
    fn add_edge(&mut self, from: BasicBlockId, to: BasicBlockId) {
        let predecessor_node = self.data.entry(from).or_default();
        if !self.reversed {
            assert!(
                predecessor_node.successors.len() < 2,
                "ICE: A cfg node cannot have more than two successors"
            );
        }
        predecessor_node.successors.insert(to);
        let successor_node = self.data.entry(to).or_default();
        successor_node.predecessors.insert(from);
    }

    /// Get an iterator over the CFG predecessors to `basic_block_id`.
    pub(crate) fn predecessors(
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
    pub(crate) fn successors(
        &self,
        basic_block_id: BasicBlockId,
    ) -> impl ExactSizeIterator<Item = BasicBlockId> + DoubleEndedIterator + '_ {
        self.data
            .get(&basic_block_id)
            .expect("ICE: Attempted to iterate successors of block not found within cfg.")
            .successors
            .iter()
            .copied()
    }

    /// Reverse the control flow graph
    pub(crate) fn reverse(&self) -> Self {
        let mut reversed_cfg = ControlFlowGraph { reversed: true, ..Default::default() };

        // Ensure the reversed_cfg always contains at least the entry node. Otherwise
        // adding the edges below results in an empty graph when calling reverse on a CFG
        // with a single block. Also note that although we are adding the entry block here,
        // it will not necessarily be the entry block in the reversed cfg.
        let entry = self.entry_block();
        reversed_cfg.data.insert(entry, Default::default());

        for (block_id, node) in &self.data {
            // For each block, reverse the edges
            // In the reversed CFG, successors becomes predecessors
            for &successor in &node.successors {
                reversed_cfg.add_edge(successor, *block_id);
            }
        }

        reversed_cfg
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
            cfg.data.keys().filter(|&&block| cfg.successors(block).len() == 0).copied().collect();
        // Traverse the reverse CFG from the exit blocks
        let reverse = cfg.reverse();
        let post_order = PostOrder::with_cfg(&reverse);

        // Extract blocks that are not reachable from the exit blocks
        let rpo_traversal: HashSet<BasicBlockId> = HashSet::from_iter(post_order.into_vec());
        let dead_blocks: Vec<BasicBlockId> =
            cfg.data.keys().filter(|&block| !rpo_traversal.contains(block)).copied().collect();

        // If some blocks, that we call 'dead' blocks, are not in the post-order traversal of the reverse CFG,
        // or if there are multiple exit nodes, then the reverse CFG is not a CFG because
        // it does not have a single entry node and so we will not be able to apply the dominance frontier algorithm.
        // In that case, we extend the CFG with a new 'exit' node and connect the exit blocks and the 'dead' blocks to it.
        if exit_nodes.len() > 1 || !dead_blocks.is_empty() {
            // Create a fake 'exit' block
            let exit = func.dfg.make_block();
            cfg.data.insert(exit, CfgNode::default());
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

    /// Return the graph of basic blocks as a petgraph graph, along with
    /// mappings to and from basicblockids and node indices.
    pub(crate) fn as_petgraph(&self) -> PetgraphCFG {
        let mut graph = petgraph::graph::DiGraph::new();
        let mut block_to_node = HashMap::default();
        let mut node_to_block = HashMap::default();

        for block_id in self.data.keys() {
            let index = graph.add_node(());
            block_to_node.insert(*block_id, index);
            node_to_block.insert(index, *block_id);
        }

        for (block_id, cfg_node) in self.data.iter() {
            let index = block_to_node[block_id];

            // We don't need to add predecessors, they'll get added when
            // we add the previous block's successors.
            for successor in cfg_node.successors.iter() {
                let successor = block_to_node[successor];
                graph.add_edge(index, successor, ());
            }
        }

        PetgraphCFG { graph, node_to_block, block_to_node }
    }

    /// Find and return the entry block
    pub(crate) fn entry_block(&self) -> BasicBlockId {
        self.data
            .iter()
            .find_map(|(block, node)| (node.predecessors.len() == 0).then_some(*block))
            .unwrap()
    }

    /// Return the topological order of blocks
    pub(crate) fn topological_order(&self) -> Vec<BasicBlockId> {
        PostOrder::compute_topological_order(self)
    }
}

/// A petgraph representation of a CFG
pub(crate) struct PetgraphCFG {
    pub(crate) graph: petgraph::graph::DiGraph<(), ()>,
    pub(crate) block_to_node: HashMap<BasicBlockId, petgraph::graph::NodeIndex>,
    pub(crate) node_to_block: HashMap<petgraph::graph::NodeIndex, BasicBlockId>,
}

#[cfg(test)]
mod tests {
    use crate::ssa::ir::{
        basic_block::BasicBlockId, instruction::TerminatorInstruction, map::Id, types::Type,
    };
    use noirc_errors::call_stack::CallStackId;

    use super::{super::function::Function, ControlFlowGraph};

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
            else_destination: block1_id,
            call_stack: CallStackId::root(),
        });
        func.dfg[block1_id].set_terminator(TerminatorInstruction::JmpIf {
            condition: cond,
            then_destination: block1_id,
            else_destination: block2_id,
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
            else_destination: ret_block_id,
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
