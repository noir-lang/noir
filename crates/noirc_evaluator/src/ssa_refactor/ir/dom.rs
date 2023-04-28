use std::{cmp::Ordering, collections::HashMap};

use self::post_order::compute_post_order;

use super::{basic_block::BasicBlockId, cfg::ControlFlowGraph, function::Function};

mod post_order;

/// Dominator tree node. We keep one of these per reachable block.
#[derive(Clone, Default)]
struct DomNode {
    /// The block's idx in the control flow graph's reverse post-order
    reverse_post_order_idx: u32,

    /// The block that immediately dominated that of the node in question.
    ///
    /// This will be None for the entry block, which has no immediate dominator.
    immediate_dominator: Option<BasicBlockId>,
}

impl DomNode {
    /// Updates the immediate dominator estimate, returning true if it has changed.
    ///
    /// This is used internally as a shorthand during `compute_dominator_tree`.
    pub(self) fn update_estimate(&mut self, immediate_dominator: BasicBlockId) -> bool {
        let immediate_dominator = Some(immediate_dominator);
        if self.immediate_dominator == immediate_dominator {
            false
        } else {
            self.immediate_dominator = immediate_dominator;
            true
        }
    }
}

/// The dominator tree for a single function.
pub(crate) struct DominatorTree {
    /// The nodes of the dominator tree
    ///
    /// After dominator tree computation has complete, this will contain a node for every
    /// reachable block, and no nodes for unreachable blocks.
    nodes: HashMap<BasicBlockId, DomNode>,

    /// CFG post-order of all reachable blocks. This is cached and exposed as it is useful for
    /// more than just computing the dominator tree.
    post_order: Vec<BasicBlockId>,
}

/// Methods for querying the dominator tree.
impl DominatorTree {
    /// Is `block_id` reachable from the entry block?
    pub(crate) fn is_reachable(&self, block_id: BasicBlockId) -> bool {
        self.nodes.contains_key(&block_id)
    }

    /// Get the CFG post-order of blocks that was used to compute the dominator tree.
    pub(crate) fn cfg_post_order(&self) -> &[BasicBlockId] {
        &self.post_order
    }

    /// Returns the immediate dominator of `block_id`.
    ///
    /// A block is said to *dominate* `block_id` if all control flow paths from the function
    /// entry to `block_id` must go through the block.
    ///
    /// The *immediate dominator* is the dominator that is closest to `block_id`. All other
    /// dominators also dominate the immediate dominator.
    ///
    /// This returns `None` if `block_id` is not reachable from the entry block, or if it is the
    /// entry block which has no dominators.
    pub(crate) fn immediate_dominator(&self, block_id: BasicBlockId) -> Option<BasicBlockId> {
        match self.nodes.get(&block_id) {
            Some(node) => node.immediate_dominator,
            _ => None,
        }
    }

    /// Compare two blocks relative to the reverse post-order.
    pub(crate) fn reverse_post_order_cmp(&self, a: BasicBlockId, b: BasicBlockId) -> Ordering {
        match (self.nodes.get(&a), self.nodes.get(&b)) {
            (Some(a), Some(b)) => a.reverse_post_order_idx.cmp(&b.reverse_post_order_idx),
            _ => unreachable!("Post order for unreachable block is undefined"),
        }
    }

    /// Returns `true` if `block_a_id` dominates `block_b_id`.
    ///
    /// This means that every control-flow path from the function entry to `block_b_id` must go
    /// through `block_a_id`.
    ///
    /// This function panics if either of the blocks are unreachable.
    ///
    /// An instruction is considered to dominate itself.
    pub(crate) fn dominates(&self, block_a_id: BasicBlockId, mut block_b_id: BasicBlockId) -> bool {
        // Walk up the dominator tree from "b" until we encounter or pass "a". Doing the
        // comparison on the reverse post-order may allows to test whether we have passed "a"
        // without waiting until we reach the root of the tree.
        loop {
            match self.reverse_post_order_cmp(block_a_id, block_b_id) {
                Ordering::Less => {
                    block_b_id = match self.immediate_dominator(block_b_id) {
                        Some(immediate_dominator) => immediate_dominator,
                        None => return false, // a is unreachable, so we climbed past the entry
                    }
                }
                Ordering::Greater => return false,
                Ordering::Equal => return true,
            }
        }
    }

    /// Compute the common dominator of two basic blocks.
    ///
    /// Both basic blocks are assumed to be reachable.
    fn common_dominator(
        &self,
        mut block_a_id: BasicBlockId,
        mut block_b_id: BasicBlockId,
    ) -> BasicBlockId {
        loop {
            match self.reverse_post_order_cmp(block_a_id, block_b_id) {
                Ordering::Less => {
                    // "a" comes before "b" in the reverse post-order. Move "b" up.
                    block_b_id = self.nodes[&block_b_id]
                        .immediate_dominator
                        .expect("Unreachable basic block?");
                }
                Ordering::Greater => {
                    // "b" comes before "a" in the reverse post-order. Move "a" up.
                    block_a_id = self.nodes[&block_a_id]
                        .immediate_dominator
                        .expect("Unreachable basic block?");
                }
                Ordering::Equal => break,
            }
        }

        debug_assert_eq!(block_a_id, block_b_id, "Unreachable block passed to common_dominator?");
        block_a_id
    }

    /// Allocate and compute a dominator tree.
    pub(crate) fn with_function(func: &Function, cfg: &ControlFlowGraph) -> Self {
        let post_order = compute_post_order(func);
        let mut domtree = DominatorTree { nodes: HashMap::new(), post_order };
        domtree.compute_dominator_tree(cfg);
        domtree
    }

    /// Build a dominator tree from a control flow graph using Keith D. Cooper's
    /// "Simple, Fast Dominator Algorithm."
    fn compute_dominator_tree(&mut self, cfg: &ControlFlowGraph) {
        // We'll be iterating over a reverse post-order of the CFG, skipping the entry block.
        let (entry_block_id, entry_free_post_order) = self
            .post_order
            .as_slice()
            .split_last()
            .expect("ICE: functions always have at least one block");

        // Do a first pass where we assign reverse post-order indices to all reachable nodes. The
        // entry block will be the only node with no immediate dominator.
        self.nodes.insert(
            *entry_block_id,
            DomNode { reverse_post_order_idx: 0, immediate_dominator: None },
        );
        for (i, &block_id) in entry_free_post_order.iter().rev().enumerate() {
            // Indices have been displaced by 1 by to the removal of the entry node
            let reverse_post_order_idx = i as u32 + 1;

            // Due to the nature of the post-order traversal, every node we visit will have at
            // least one predecessor that has previously been assigned during this loop.
            let immediate_dominator = self.compute_immediate_dominator(block_id, cfg);
            self.nodes.insert(
                block_id,
                DomNode { immediate_dominator: Some(immediate_dominator), reverse_post_order_idx },
            );
        }

        // Now that we have reverse post-order indices for everything and initial immediate
        // dominator estimates, iterate until convergence.
        //
        // If the function is free of irreducible control flow, this will exit after one iteration.
        let mut changed = true;
        while changed {
            changed = false;
            for &block_id in entry_free_post_order.iter().rev() {
                let immediate_dominator = self.compute_immediate_dominator(block_id, cfg);
                changed = self
                    .nodes
                    .get_mut(&block_id)
                    .expect("Assigned in first pass")
                    .update_estimate(immediate_dominator);
            }
        }
    }

    // Compute the immediate dominator for `block_id` using the pre-calculate immediate dominators
    // of reachable nodes.
    fn compute_immediate_dominator(
        &self,
        block_id: BasicBlockId,
        cfg: &ControlFlowGraph,
    ) -> BasicBlockId {
        // Get an iterator with just the reachable, already visited predecessors to `block_id`.
        // Note that during the first pass `node` was pre-populated with all reachable blocks.
        let mut reachable_preds =
            cfg.pred_iter(block_id).filter(|pred_id| self.nodes.contains_key(&pred_id));

        // This function isn't called on unreachable blocks or the entry block, so the reverse
        // post-order will contain at least one predecessor to this block.
        let mut immediate_dominator =
            reachable_preds.next().expect("block node must have one reachable predecessor");

        for pred in reachable_preds {
            immediate_dominator = self.common_dominator(immediate_dominator, pred);
        }

        immediate_dominator
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use crate::ssa_refactor::ir::{
        basic_block::BasicBlockId, cfg::ControlFlowGraph, dom::DominatorTree, function::Function,
        instruction::TerminatorInstruction, map::Id, types::Type,
    };

    #[test]
    fn empty() {
        let func_id = Id::test_new(0);
        let mut func = Function::new("func".into(), func_id);
        let block0_id = func.entry_block();
        func.dfg.set_block_terminator(
            block0_id,
            TerminatorInstruction::Return { return_values: vec![] },
        );
        let cfg = ControlFlowGraph::with_function(&func);
        let dom_tree = DominatorTree::with_function(&func, &cfg);
        assert_eq!(dom_tree.cfg_post_order(), &[block0_id]);
    }

    // Testing setup for a function with an unreachable block2
    fn unreachable_node_setup(
    ) -> (DominatorTree, BasicBlockId, BasicBlockId, BasicBlockId, BasicBlockId) {
        // func() {
        //   block0(cond: u1):
        //     jmpif v0 block2() block3()
        //   block1():
        //     jmp block2()
        //   block2():
        //     jmp block3()
        //   block3():
        //     return ()
        // }
        let func_id = Id::test_new(0);
        let mut func = Function::new("func".into(), func_id);
        let block0_id = func.entry_block();
        let block1_id = func.dfg.make_block();
        let block2_id = func.dfg.make_block();
        let block3_id = func.dfg.make_block();

        let cond = func.dfg.add_block_parameter(block0_id, Type::unsigned(1));
        func.dfg.set_block_terminator(
            block0_id,
            TerminatorInstruction::JmpIf {
                condition: cond,
                then_destination: block2_id,
                else_destination: block3_id,
            },
        );
        func.dfg.set_block_terminator(
            block1_id,
            TerminatorInstruction::Jmp { destination: block2_id, arguments: vec![] },
        );
        func.dfg.set_block_terminator(
            block2_id,
            TerminatorInstruction::Jmp { destination: block3_id, arguments: vec![] },
        );
        func.dfg.set_block_terminator(
            block3_id,
            TerminatorInstruction::Return { return_values: vec![] },
        );

        let cfg = ControlFlowGraph::with_function(&func);
        let dt = DominatorTree::with_function(&func, &cfg);
        (dt, block0_id, block1_id, block2_id, block3_id)
    }

    // Expected dominator tree
    // block0 {
    //   block2
    //   block3
    // }

    // Dominance matrix
    // ✓: Row item dominates column item
    // !: Querying row item's dominance of column item panics. (i.e. invalid)
    //    b0  b1  b2  b3
    // b0 ✓   !   ✓   ✓
    // b1 !   !   !   !
    // b2     !   ✓
    // b3     !       ✓
    // Note that from a local view block 1 dominates blocks 1,2 & 3, but since this block is
    // unreachable, performing this query indicates an internal compiler error.
    #[test]
    fn unreachable_node_asserts() {
        let (dt, b0, _b1, b2, b3) = unreachable_node_setup();

        assert_eq!(dt.cfg_post_order(), &[b3, b2, b0]);

        assert_eq!(dt.dominates(b0, b0), true);
        assert_eq!(dt.dominates(b0, b2), true);
        assert_eq!(dt.dominates(b0, b3), true);

        assert_eq!(dt.dominates(b2, b0), false);
        assert_eq!(dt.dominates(b2, b2), true);
        assert_eq!(dt.dominates(b2, b3), false);

        assert_eq!(dt.dominates(b3, b0), false);
        assert_eq!(dt.dominates(b3, b2), false);
        assert_eq!(dt.dominates(b3, b3), true);
    }

    #[test]
    #[should_panic]
    fn unreachable_node_panic_b0_b1() {
        let (dt, b0, b1, _b2, _b3) = unreachable_node_setup();
        dt.dominates(b0, b1);
    }

    #[test]
    #[should_panic]
    fn unreachable_node_panic_b1_b0() {
        let (dt, b0, b1, _b2, _b3) = unreachable_node_setup();
        dt.dominates(b1, b0);
    }

    #[test]
    #[should_panic]
    fn unreachable_node_panic_b1_b1() {
        let (dt, _b0, b1, _b2, _b3) = unreachable_node_setup();
        dt.dominates(b1, b1);
    }

    #[test]
    #[should_panic]
    fn unreachable_node_panic_b1_b2() {
        let (dt, _b0, b1, b2, _b3) = unreachable_node_setup();
        dt.dominates(b1, b2);
    }

    #[test]
    #[should_panic]
    fn unreachable_node_panic_b1_b3() {
        let (dt, _b0, b1, _b2, b3) = unreachable_node_setup();
        dt.dominates(b1, b3);
    }

    #[test]
    #[should_panic]
    fn unreachable_node_panic_b3_b1() {
        let (dt, _b0, b1, b2, _b3) = unreachable_node_setup();
        dt.dominates(b2, b1);
    }

    #[test]
    fn backwards_layout() {
        // func {
        //   block0():
        //     jmp block2()
        //   block1():
        //     return ()
        //   block2():
        //     jump block1()
        // }
        let func_id = Id::test_new(0);
        let mut func = Function::new("func".into(), func_id);
        let block0_id = func.entry_block();
        let block1_id = func.dfg.make_block();
        let block2_id = func.dfg.make_block();

        func.dfg.set_block_terminator(
            block0_id,
            TerminatorInstruction::Jmp { destination: block2_id, arguments: vec![] },
        );
        func.dfg.set_block_terminator(
            block1_id,
            TerminatorInstruction::Return { return_values: vec![] },
        );
        func.dfg.set_block_terminator(
            block2_id,
            TerminatorInstruction::Jmp { destination: block1_id, arguments: vec![] },
        );

        let cfg = ControlFlowGraph::with_function(&func);
        let dt = DominatorTree::with_function(&func, &cfg);

        // Expected dominance tree:
        // block0 {
        //   block2 {
        //     block1
        //   }
        // }

        assert_eq!(dt.immediate_dominator(block0_id), None);
        assert_eq!(dt.immediate_dominator(block1_id), Some(block2_id));
        assert_eq!(dt.immediate_dominator(block2_id), Some(block0_id));

        assert_eq!(dt.reverse_post_order_cmp(block0_id, block0_id), Ordering::Equal);
        assert_eq!(dt.reverse_post_order_cmp(block0_id, block1_id), Ordering::Less);
        assert_eq!(dt.reverse_post_order_cmp(block0_id, block2_id), Ordering::Less);

        assert_eq!(dt.reverse_post_order_cmp(block1_id, block0_id), Ordering::Greater);
        assert_eq!(dt.reverse_post_order_cmp(block1_id, block1_id), Ordering::Equal);
        assert_eq!(dt.reverse_post_order_cmp(block1_id, block2_id), Ordering::Greater);

        assert_eq!(dt.reverse_post_order_cmp(block2_id, block0_id), Ordering::Greater);
        assert_eq!(dt.reverse_post_order_cmp(block2_id, block1_id), Ordering::Less);
        assert_eq!(dt.reverse_post_order_cmp(block2_id, block2_id), Ordering::Equal);

        // Dominance matrix:
        // ✓: Row item dominates column item
        //    b0  b1  b2
        // b0 ✓   ✓   ✓
        // b1     ✓
        // b2     ✓   ✓

        assert_eq!(dt.dominates(block0_id, block0_id), true);
        assert_eq!(dt.dominates(block0_id, block1_id), true);
        assert_eq!(dt.dominates(block0_id, block2_id), true);

        assert_eq!(dt.dominates(block1_id, block0_id), false);
        assert_eq!(dt.dominates(block1_id, block1_id), true);
        assert_eq!(dt.dominates(block1_id, block2_id), false);

        assert_eq!(dt.dominates(block2_id, block0_id), false);
        assert_eq!(dt.dominates(block2_id, block1_id), true);
        assert_eq!(dt.dominates(block2_id, block2_id), true);
    }
}
