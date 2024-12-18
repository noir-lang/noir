//! The dominator tree of a function, represented as a hash map of each reachable block id to its
//! immediate dominator.
//!
//! Dominator trees are useful for tasks such as identifying back-edges in loop analysis or
//! calculating dominance frontiers.

use std::cmp::Ordering;

use super::{
    basic_block::BasicBlockId, cfg::ControlFlowGraph, function::Function, post_order::PostOrder,
};
use fxhash::FxHashMap as HashMap;

/// Dominator tree node. We keep one of these per reachable block.
#[derive(Clone, Default)]
struct DominatorTreeNode {
    /// The block's idx in the control flow graph's reverse post-order
    reverse_post_order_idx: u32,

    /// The block that immediately dominated that of the node in question.
    ///
    /// This will be None for the entry block, which has no immediate dominator.
    immediate_dominator: Option<BasicBlockId>,
}

impl DominatorTreeNode {
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
    nodes: HashMap<BasicBlockId, DominatorTreeNode>,

    /// Subsequent calls to `dominates` are cached to speed up access
    cache: HashMap<(BasicBlockId, BasicBlockId), bool>,
}

/// Methods for querying the dominator tree.
impl DominatorTree {
    /// Is `block_id` reachable from the entry block?
    pub(crate) fn is_reachable(&self, block_id: BasicBlockId) -> bool {
        self.nodes.contains_key(&block_id)
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
        self.nodes.get(&block_id).and_then(|node| node.immediate_dominator)
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
    /// A block is considered to dominate itself.
    pub(crate) fn dominates(&mut self, block_a_id: BasicBlockId, block_b_id: BasicBlockId) -> bool {
        if let Some(res) = self.cache.get(&(block_a_id, block_b_id)) {
            return *res;
        }

        let result = self.dominates_helper(block_a_id, block_b_id);
        self.cache.insert((block_a_id, block_b_id), result);
        result
    }

    pub(crate) fn dominates_helper(
        &self,
        block_a_id: BasicBlockId,
        mut block_b_id: BasicBlockId,
    ) -> bool {
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

    /// Walk up the dominator tree until we find a block for which `f` returns `Some` value.
    /// Otherwise return `None` when we reach the top.
    ///
    /// Similar to `Iterator::filter_map` but only returns the first hit.
    pub(crate) fn find_map_dominator<T>(
        &self,
        mut block_id: BasicBlockId,
        f: impl Fn(BasicBlockId) -> Option<T>,
    ) -> Option<T> {
        if !self.is_reachable(block_id) {
            return None;
        }
        loop {
            if let Some(value) = f(block_id) {
                return Some(value);
            }
            block_id = match self.immediate_dominator(block_id) {
                Some(immediate_dominator) => immediate_dominator,
                None => return None,
            }
        }
    }

    /// Allocate and compute a dominator tree from a pre-computed control flow graph and
    /// post-order counterpart.
    pub(crate) fn with_cfg_and_post_order(cfg: &ControlFlowGraph, post_order: &PostOrder) -> Self {
        let mut dom_tree = DominatorTree { nodes: HashMap::default(), cache: HashMap::default() };
        dom_tree.compute_dominator_tree(cfg, post_order);
        dom_tree
    }

    /// Allocate and compute a dominator tree for the given function.
    ///
    /// This approach computes the control flow graph and post-order internally and then
    /// discards them. If either should be retained reuse it is better to instead pre-compute them
    /// and build the dominator tree with `DominatorTree::with_cfg_and_post_order`.
    pub(crate) fn with_function(func: &Function) -> Self {
        let cfg = ControlFlowGraph::with_function(func);
        let post_order = PostOrder::with_function(func);
        Self::with_cfg_and_post_order(&cfg, &post_order)
    }

    /// Build a dominator tree from a control flow graph using Keith D. Cooper's
    /// "Simple, Fast Dominator Algorithm."
    fn compute_dominator_tree(&mut self, cfg: &ControlFlowGraph, post_order: &PostOrder) {
        // We'll be iterating over a reverse post-order of the CFG, skipping the entry block.
        let (entry_block_id, entry_free_post_order) = post_order
            .as_slice()
            .split_last()
            .expect("ICE: functions always have at least one block");

        // Do a first pass where we assign reverse post-order indices to all reachable nodes. The
        // entry block will be the only node with no immediate dominator.
        self.nodes.insert(
            *entry_block_id,
            DominatorTreeNode { reverse_post_order_idx: 0, immediate_dominator: None },
        );
        for (i, &block_id) in entry_free_post_order.iter().rev().enumerate() {
            // Indices have been displaced by 1 by the removal of the entry node
            let reverse_post_order_idx = i as u32 + 1;

            // Due to the nature of the post-order traversal, every node we visit will have at
            // least one predecessor that has previously been assigned during this loop.
            let immediate_dominator = self.compute_immediate_dominator(block_id, cfg);
            self.nodes.insert(
                block_id,
                DominatorTreeNode {
                    immediate_dominator: Some(immediate_dominator),
                    reverse_post_order_idx,
                },
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
        let mut reachable_predecessors =
            cfg.predecessors(block_id).filter(|pred_id| self.nodes.contains_key(pred_id));

        // This function isn't called on unreachable blocks or the entry block, so the reverse
        // post-order will contain at least one predecessor to this block.
        let mut immediate_dominator =
            reachable_predecessors.next().expect("block node must have one reachable predecessor");

        for predecessor in reachable_predecessors {
            immediate_dominator = self.common_dominator(immediate_dominator, predecessor);
        }

        immediate_dominator
    }

    /// Compute the common dominator of two basic blocks.
    ///
    /// Both basic blocks are assumed to be reachable.
    pub(crate) fn common_dominator(
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
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            basic_block::BasicBlockId, call_stack::CallStackId, dom::DominatorTree,
            function::Function, instruction::TerminatorInstruction, map::Id, types::Type,
        },
    };

    #[test]
    fn empty() {
        let func_id = Id::test_new(0);
        let mut func = Function::new("func".into(), func_id);
        let block0_id = func.entry_block();
        func.dfg.set_block_terminator(
            block0_id,
            TerminatorInstruction::Return {
                return_values: vec![],
                call_stack: CallStackId::root(),
            },
        );
        let mut dom_tree = DominatorTree::with_function(&func);
        assert!(dom_tree.dominates(block0_id, block0_id));
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
        let mut builder = FunctionBuilder::new("func".into(), func_id);

        let cond = builder.add_parameter(Type::unsigned(1));
        let block1_id = builder.insert_block();
        let block2_id = builder.insert_block();
        let block3_id = builder.insert_block();

        builder.terminate_with_jmpif(cond, block2_id, block3_id);
        builder.switch_to_block(block1_id);
        builder.terminate_with_jmp(block2_id, vec![]);
        builder.switch_to_block(block2_id);
        builder.terminate_with_jmp(block3_id, vec![]);
        builder.switch_to_block(block3_id);
        builder.terminate_with_return(vec![]);

        let ssa = builder.finish();
        let func = ssa.main();
        let block0_id = func.entry_block();

        let dt = DominatorTree::with_function(func);
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
        let (mut dt, b0, _b1, b2, b3) = unreachable_node_setup();

        assert!(dt.dominates(b0, b0));
        assert!(dt.dominates(b0, b2));
        assert!(dt.dominates(b0, b3));

        assert!(!dt.dominates(b2, b0));
        assert!(dt.dominates(b2, b2));
        assert!(!dt.dominates(b2, b3));

        assert!(!dt.dominates(b3, b0));
        assert!(!dt.dominates(b3, b2));
        assert!(dt.dominates(b3, b3));
    }

    #[test]
    #[should_panic]
    fn unreachable_node_panic_b0_b1() {
        let (mut dt, b0, b1, _b2, _b3) = unreachable_node_setup();
        dt.dominates(b0, b1);
    }

    #[test]
    #[should_panic]
    fn unreachable_node_panic_b1_b0() {
        let (mut dt, b0, b1, _b2, _b3) = unreachable_node_setup();
        dt.dominates(b1, b0);
    }

    #[test]
    #[should_panic]
    fn unreachable_node_panic_b1_b1() {
        let (mut dt, _b0, b1, _b2, _b3) = unreachable_node_setup();
        dt.dominates(b1, b1);
    }

    #[test]
    #[should_panic]
    fn unreachable_node_panic_b1_b2() {
        let (mut dt, _b0, b1, b2, _b3) = unreachable_node_setup();
        dt.dominates(b1, b2);
    }

    #[test]
    #[should_panic]
    fn unreachable_node_panic_b1_b3() {
        let (mut dt, _b0, b1, _b2, b3) = unreachable_node_setup();
        dt.dominates(b1, b3);
    }

    #[test]
    #[should_panic]
    fn unreachable_node_panic_b3_b1() {
        let (mut dt, _b0, b1, b2, _b3) = unreachable_node_setup();
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
        let mut builder = FunctionBuilder::new("func".into(), func_id);
        let block1_id = builder.insert_block();
        let block2_id = builder.insert_block();

        builder.terminate_with_jmp(block2_id, vec![]);
        builder.switch_to_block(block1_id);
        builder.terminate_with_return(vec![]);
        builder.switch_to_block(block2_id);
        builder.terminate_with_jmp(block1_id, vec![]);

        let ssa = builder.finish();
        let func = ssa.main();
        let block0_id = func.entry_block();

        let mut dt = DominatorTree::with_function(func);

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

        assert!(dt.dominates(block0_id, block0_id));
        assert!(dt.dominates(block0_id, block1_id));
        assert!(dt.dominates(block0_id, block2_id));

        assert!(!dt.dominates(block1_id, block0_id));
        assert!(dt.dominates(block1_id, block1_id));
        assert!(!dt.dominates(block1_id, block2_id));

        assert!(!dt.dominates(block2_id, block0_id));
        assert!(dt.dominates(block2_id, block1_id));
        assert!(dt.dominates(block2_id, block2_id));
    }

    #[test]
    fn test_find_map_dominator() {
        let (dt, b0, b1, b2, _b3) = unreachable_node_setup();

        assert_eq!(
            dt.find_map_dominator(b2, |b| if b == b0 { Some("root") } else { None }),
            Some("root")
        );
        assert_eq!(
            dt.find_map_dominator(b1, |b| if b == b0 { Some("unreachable") } else { None }),
            None
        );
        assert_eq!(
            dt.find_map_dominator(b1, |b| if b == b1 { Some("not part of tree") } else { None }),
            None
        );
    }
}
