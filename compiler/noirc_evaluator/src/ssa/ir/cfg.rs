use std::collections::BTreeSet;

use super::{
    basic_block::{BasicBlock, BasicBlockId},
    function::Function,
};
use fxhash::FxHashMap as HashMap;

/// A container for the successors and predecessors of some Block.
#[derive(Clone, Default)]
struct CfgNode {
    /// Set of blocks that containing jumps that target this block.
    /// The predecessor set has no meaningful order.
    pub(crate) predecessors: BTreeSet<BasicBlockId>,

    /// Set of blocks that are the targets of jumps in this block.
    /// The successors set has no meaningful order.
    pub(crate) successors: BTreeSet<BasicBlockId>,
}

#[derive(Clone)]
/// The Control Flow Graph maintains a mapping of blocks to their predecessors
/// and successors where predecessors are basic blocks and successors are
/// basic blocks.
pub(crate) struct ControlFlowGraph {
    data: HashMap<BasicBlockId, CfgNode>,
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

        let mut cfg = ControlFlowGraph { data };
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
        assert!(
            predecessor_node.successors.len() < 2,
            "ICE: A cfg node cannot have more than two successors"
        );
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
    use crate::ssa::ir::{
        dfg::CallStack, instruction::TerminatorInstruction, map::Id, types::Type,
    };

    use super::{super::function::Function, ControlFlowGraph};

    #[test]
    fn empty() {
        let func_id = Id::test_new(0);
        let mut func = Function::new("func".into(), func_id);
        let block_id = func.entry_block();
        func.dfg[block_id].set_terminator(TerminatorInstruction::Return {
            return_values: vec![],
            call_stack: CallStack::new(),
        });

        ControlFlowGraph::with_function(&func);
    }

    #[test]
    fn jumps() {
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
            call_stack: CallStack::new(),
        });
        func.dfg[block1_id].set_terminator(TerminatorInstruction::JmpIf {
            condition: cond,
            then_destination: block1_id,
            else_destination: block2_id,
            call_stack: CallStack::new(),
        });
        func.dfg[block2_id].set_terminator(TerminatorInstruction::Return {
            return_values: vec![],
            call_stack: CallStack::new(),
        });

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
            call_stack: CallStack::new(),
        });
        func.dfg[block2_id].set_terminator(TerminatorInstruction::Jmp {
            destination: ret_block_id,
            arguments: vec![],
            call_stack: im::Vector::new(),
        });
        func.dfg[block0_id].set_terminator(TerminatorInstruction::JmpIf {
            condition: cond,
            then_destination: block1_id,
            else_destination: ret_block_id,
            call_stack: CallStack::new(),
        });

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
}
