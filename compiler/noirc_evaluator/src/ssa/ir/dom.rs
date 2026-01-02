//! The dominator tree of a function, represented as a hash map of each reachable block id to its
//! immediate dominator.
//!
//! Dominator trees are useful for tasks such as identifying back-edges in loop analysis or
//! calculating dominance frontiers.

use std::cmp::Ordering;

use super::{
    basic_block::BasicBlockId, cfg::ControlFlowGraph, function::Function, post_order::PostOrder,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

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
#[derive(Default)]
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
        match (self.reverse_post_order_idx(a), self.reverse_post_order_idx(b)) {
            (Some(a), Some(b)) => a.cmp(&b),
            _ => unreachable!("Post order for unreachable block is undefined"),
        }
    }

    /// Position in the Reverse Post-Order.
    pub(crate) fn reverse_post_order_idx(&self, block_id: BasicBlockId) -> Option<u32> {
        self.nodes.get(&block_id).map(|n| n.reverse_post_order_idx)
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
        // comparison on the reverse post-order may allow to test whether we have passed "a"
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
            block_id = self.immediate_dominator(block_id)?;
        }
    }

    /// Allocate and compute a dominator tree from a pre-computed control flow graph and
    /// post-order counterpart.
    ///
    /// This method should be used for when we want to compute a post-dominator tree.
    /// A post-dominator tree just expects the control flow graph to be reversed.
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

    /// Allocate and compute a post-dominator tree for the given function.
    ///
    /// This approach computes the reversed control flow graph and post-order internally and then
    /// discards them. If either should be retained for reuse, it is better to instead pre-compute them
    /// and build the dominator tree with `DominatorTree::with_cfg_and_post_order`.
    #[cfg(test)]
    pub(crate) fn with_function_post_dom(func: &Function) -> Self {
        let reversed_cfg = ControlFlowGraph::with_function(func).reverse();
        let post_order = PostOrder::with_cfg(&reversed_cfg);
        Self::with_cfg_and_post_order(&reversed_cfg, &post_order)
    }

    /// Build a dominator tree from a control flow graph using Keith D. Cooper's
    /// "Simple, Fast Dominator Algorithm."
    fn compute_dominator_tree(&mut self, cfg: &ControlFlowGraph, post_order: &PostOrder) {
        // We'll be iterating over a reverse post-order of the CFG, skipping the entry block.
        let Some((entry_block_id, entry_free_post_order)) = post_order.as_slice().split_last()
        else {
            return;
        };

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

    /// Computes the dominance frontier for all blocks in the dominator tree.
    ///
    /// The Dominance Frontier of a basic block X is the set of all blocks that are immediate
    /// successors to blocks dominated by X, but which aren’t themselves strictly dominated by X.
    /// It is the set of blocks that are not dominated X, and which are “first reached” on paths from X.
    ///
    /// For example in the following CFG the DF of B is {E}, because B dominates {C},
    /// but it's just one edge away from dominating E, as there is another path to E through D.
    /// ```text
    ///    A
    ///   / \
    ///  B   D
    ///  |   |
    ///  C   |
    ///   \ /
    ///    E
    /// ```
    ///
    /// This method uses the algorithm specified in Cooper, Keith D. et al. “A Simple, Fast Dominance Algorithm.” (1999).
    /// As referenced in the paper a dominance frontier is the set of all CFG nodes, y, such that
    /// b dominates a predecessor of y but does not strictly dominate y.
    ///
    /// This method expects the appropriate CFG depending on whether we are operating over
    /// a dominator tree (standard CFG) or a post-dominator tree (reversed CFG).
    /// Calling this method on a dominator tree will return a function's dominance frontiers,
    /// while on a post-dominator tree the method will return the function's reverse (or post) dominance frontiers.
    pub(crate) fn compute_dominance_frontiers(
        &mut self,
        cfg: &ControlFlowGraph,
    ) -> HashMap<BasicBlockId, HashSet<BasicBlockId>> {
        let mut dominance_frontiers: HashMap<BasicBlockId, HashSet<BasicBlockId>> =
            HashMap::default();

        let nodes = self.nodes.keys().copied().collect::<Vec<_>>();
        // Find out about each block which dominance frontiers they belong to, if any.
        for block_id in nodes {
            let predecessors = cfg.predecessors(block_id);
            // Dominance frontier nodes must have more than one predecessor. They are join points in the CFG.
            if predecessors.len() <= 1 {
                continue;
            }
            let Some(immediate_dominator) = self.immediate_dominator(block_id) else {
                continue;
            };
            // Iterate over the predecessors of the current block and walk backwards from them in the dominator tree.
            for pred_id in predecessors {
                let mut runner = pred_id;
                loop {
                    // Once we reach the immediate dominator of the current block, we know the current block
                    // won't be in the frontier of any further blocks (frontier blocks are *not* dominated by them).
                    if immediate_dominator == runner {
                        break;
                    }
                    // Checking if the current block dominates the predecessor;
                    // for example a loop header has the loop body as one of its predecessors, which it dominates,
                    // but we don't consider following back-edges as alternative paths on which we reach the header first.
                    if self.dominates(block_id, runner) {
                        break;
                    }
                    dominance_frontiers.entry(runner).or_default().insert(block_id);
                    // Continue walking backwards to the dominators of the runner, which also have the
                    // current block in their frontier, unless they dominate it.
                    let Some(runner_immediate_dom) = self.immediate_dominator(runner) else {
                        break;
                    };
                    runner = runner_immediate_dom;
                }
            }
        }

        dominance_frontiers
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use iter_extended::vecmap;
    use noirc_errors::call_stack::CallStackId;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            basic_block::{BasicBlock, BasicBlockId},
            cfg::ControlFlowGraph,
            dom::DominatorTree,
            function::Function,
            instruction::TerminatorInstruction,
            map::Id,
            post_order::PostOrder,
            types::Type,
        },
        ssa_gen::Ssa,
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
    fn unreachable_node_setup()
    -> (DominatorTree, BasicBlockId, BasicBlockId, BasicBlockId, BasicBlockId) {
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

    fn backwards_layout_setup() -> Function {
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
        ssa.main().clone()
    }

    fn check_dom_matrix(
        mut dom_tree: DominatorTree,
        blocks: Vec<BasicBlockId>,
        dominance_matrix: Vec<Vec<bool>>,
    ) {
        for (i, row) in dominance_matrix.into_iter().enumerate() {
            for (j, expected) in row.into_iter().enumerate() {
                assert_eq!(dom_tree.dominates(blocks[i], blocks[j]), expected);
            }
        }
    }

    #[test]
    fn backwards_layout() {
        let func = backwards_layout_setup();
        let dt = DominatorTree::with_function(&func);

        // Expected dominance tree:
        // block0 {
        //   block2 {
        //     block1
        //   }
        // }

        let blocks = vecmap(0..3, Id::<BasicBlock>::test_new);

        assert_eq!(dt.immediate_dominator(blocks[0]), None);
        assert_eq!(dt.immediate_dominator(blocks[1]), Some(blocks[2]));
        assert_eq!(dt.immediate_dominator(blocks[2]), Some(blocks[0]));

        assert_eq!(dt.reverse_post_order_cmp(blocks[0], blocks[0]), Ordering::Equal);
        assert_eq!(dt.reverse_post_order_cmp(blocks[0], blocks[1]), Ordering::Less);
        assert_eq!(dt.reverse_post_order_cmp(blocks[0], blocks[2]), Ordering::Less);

        assert_eq!(dt.reverse_post_order_cmp(blocks[1], blocks[0]), Ordering::Greater);
        assert_eq!(dt.reverse_post_order_cmp(blocks[1], blocks[1]), Ordering::Equal);
        assert_eq!(dt.reverse_post_order_cmp(blocks[1], blocks[2]), Ordering::Greater);

        assert_eq!(dt.reverse_post_order_cmp(blocks[2], blocks[0]), Ordering::Greater);
        assert_eq!(dt.reverse_post_order_cmp(blocks[2], blocks[1]), Ordering::Less);
        assert_eq!(dt.reverse_post_order_cmp(blocks[2], blocks[2]), Ordering::Equal);

        // Dominance matrix:
        // ✓: Row item dominates column item
        //    b0  b1  b2
        // b0 ✓   ✓   ✓
        // b1     ✓
        // b2     ✓   ✓

        let dominance_matrix =
            vec![vec![true, true, true], vec![false, true, false], vec![false, true, true]];

        check_dom_matrix(dt, blocks, dominance_matrix);
    }

    #[test]
    fn post_dom_backwards_layout() {
        let func = backwards_layout_setup();
        let post_dom = DominatorTree::with_function_post_dom(&func);

        // Expected post-dominator tree:
        // block1 {
        //   block2 {
        //     block0
        //   }
        // }

        let blocks = vecmap(0..3, Id::<BasicBlock>::test_new);

        assert_eq!(post_dom.immediate_dominator(blocks[0]), Some(blocks[2]));
        assert_eq!(post_dom.immediate_dominator(blocks[1]), None);
        assert_eq!(post_dom.immediate_dominator(blocks[2]), Some(blocks[1]));

        assert_eq!(post_dom.reverse_post_order_cmp(blocks[0], blocks[0]), Ordering::Equal);
        assert_eq!(post_dom.reverse_post_order_cmp(blocks[0], blocks[1]), Ordering::Greater);
        assert_eq!(post_dom.reverse_post_order_cmp(blocks[0], blocks[2]), Ordering::Greater);

        assert_eq!(post_dom.reverse_post_order_cmp(blocks[1], blocks[0]), Ordering::Less);
        assert_eq!(post_dom.reverse_post_order_cmp(blocks[1], blocks[1]), Ordering::Equal);
        assert_eq!(post_dom.reverse_post_order_cmp(blocks[1], blocks[2]), Ordering::Less);

        assert_eq!(post_dom.reverse_post_order_cmp(blocks[2], blocks[0]), Ordering::Less);
        assert_eq!(post_dom.reverse_post_order_cmp(blocks[2], blocks[1]), Ordering::Greater);
        assert_eq!(post_dom.reverse_post_order_cmp(blocks[2], blocks[2]), Ordering::Equal);

        // Post-dominance matrix:
        // ✓: Row item post-dominates column item
        //    b0  b1  b2
        // b0 ✓
        // b1 ✓   ✓   ✓
        // b2 ✓       ✓

        let post_dominance_matrix =
            vec![vec![true, false, false], vec![true, true, true], vec![true, false, true]];

        check_dom_matrix(post_dom, blocks, post_dominance_matrix);
    }

    #[test]
    fn dom_frontiers_backwards_layout() {
        let func = backwards_layout_setup();
        let mut dt = DominatorTree::with_function(&func);

        let cfg = ControlFlowGraph::with_function(&func);
        let dom_frontiers = dt.compute_dominance_frontiers(&cfg);
        assert!(dom_frontiers.is_empty());
    }

    #[test]
    fn post_dom_frontiers_backwards_layout() {
        let func = backwards_layout_setup();
        let mut post_dom = DominatorTree::with_function_post_dom(&func);

        let cfg = ControlFlowGraph::with_function(&func);
        let dom_frontiers = post_dom.compute_dominance_frontiers(&cfg);
        assert!(dom_frontiers.is_empty());
    }

    /// ```text
    ///       b0
    ///       |
    /// +---> b1
    /// |    /  \
    /// |   b2  b3
    /// |  / |
    /// | b4 |
    /// |  \ |
    /// +---b5
    /// ```
    fn loop_with_cond() -> Ssa {
        let src = "
        brillig(inline) fn main f0 {
          b0(v1: u32, v2: u32):
            v5 = eq v1, u32 5
            jmp b1(u32 0)
          b1(v3: u32):
            v8 = lt v3, u32 4
            jmpif v8 then: b2, else: b3
          b2():
            jmpif v5 then: b4, else: b5
          b3():
            return
          b4():
            v9 = mul u32 4294967295, v2
            constrain v9 == u32 12
            jmp b5()
          b5():
            v12 = unchecked_add v3, u32 1
            jmp b1(v12)
        }
        ";
        Ssa::from_str(src).unwrap()
    }

    #[test]
    fn dom_loop_with_cond() {
        let ssa = loop_with_cond();
        let main = ssa.main();
        let dt = DominatorTree::with_function(main);

        let blocks = vecmap(0..6, Id::<BasicBlock>::test_new);
        // Dominance matrix:
        // ✓: Row item dominates column item
        //    b0  b1  b2  b3  b4  b5
        // b0 ✓   ✓   ✓   ✓   ✓   ✓
        // b1     ✓   ✓   ✓   ✓   ✓
        // b2         ✓       ✓   ✓
        // b3             ✓
        // b4                 ✓
        // b5                     ✓

        let dominance_matrix = vec![
            vec![true, true, true, true, true, true],
            vec![false, true, true, true, true, true],
            vec![false, false, true, false, true, true],
            vec![false, false, false, true, false, false],
            vec![false, false, false, false, true, false],
            vec![false, false, false, false, false, true],
        ];

        check_dom_matrix(dt, blocks, dominance_matrix);
    }

    #[test]
    fn post_dom_loop_with_cond() {
        let ssa = loop_with_cond();
        let main = ssa.main();

        let cfg = ControlFlowGraph::with_function(main);
        let reversed_cfg = cfg.reverse();
        let post_order = PostOrder::with_cfg(&reversed_cfg);

        let post_dom = DominatorTree::with_cfg_and_post_order(&reversed_cfg, &post_order);

        let blocks = vecmap(0..6, Id::<BasicBlock>::test_new);

        // b0 is the entry node, thus it does not post-dominate anything except itself
        //
        // b2 and b4 are leaves in the post-dominator tree. There are no nodes that must pass through
        // those blocks to reach the exit node.
        // The dominator tree computation does not recognize that the loop has constant bounds,
        // so it will still account for the jmpif in b1 and the possibility of skipping b2.
        //
        // All nodes except the exit node b3, must pass through b1 to reach the exit node.
        //
        // Starting from the exit node b3 which should be the root of the post-dominator tree
        // Every block except for the loop header b1, the exit node b3, and the entry node b0,
        // must pass through the loop exit, b5, to reach the exit node.
        //
        // Post-dominance matrix:
        // ✓: Row item post-dominates column item
        //    b0  b1  b2  b3  b4  b5
        // b0 ✓
        // b1 ✓   ✓   ✓       ✓   ✓
        // b2         ✓
        // b3 ✓   ✓   ✓   ✓   ✓   ✓
        // b4                 ✓
        // b5         ✓       ✓   ✓

        let post_dominance_matrix = vec![
            vec![true, false, false, false, false, false],
            vec![true, true, true, false, true, true],
            vec![false, false, true, false, false, false],
            vec![true, true, true, true, true, true],
            vec![false, false, false, false, true, false],
            vec![false, false, true, false, true, true],
        ];

        check_dom_matrix(post_dom, blocks, post_dominance_matrix);
    }

    #[test]
    fn dom_frontiers() {
        let ssa = loop_with_cond();
        let main = ssa.main();

        let cfg = ControlFlowGraph::with_function(main);
        let post_order = PostOrder::with_cfg(&cfg);

        let mut dt = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);
        let dom_frontiers = dt.compute_dominance_frontiers(&cfg);

        let blocks = vecmap(0..6, Id::<BasicBlock>::test_new);

        // b0 is the entry block which dominates all other blocks
        // Thus, it has an empty set for its dominance frontier
        assert!(!dom_frontiers.contains_key(&blocks[0]));
        assert!(!dom_frontiers.contains_key(&blocks[1]));
        assert!(!dom_frontiers.contains_key(&blocks[2]));
        // b3 is the exit block which does not dominate any blocks
        assert!(!dom_frontiers.contains_key(&blocks[3]));

        // b4 has DF { b5 } because b4 jumps to b5, thus being a predecessor to b5.
        // b5 dominates itself but b5 does not strictly dominate b4.
        let b4_df = &dom_frontiers[&blocks[4]];
        assert_eq!(b4_df.len(), 1);
        assert!(b4_df.contains(&blocks[5]));

        assert!(!dom_frontiers.contains_key(&blocks[5]));
    }

    #[test]
    fn dom_frontiers_not_include_self() {
        // In this example b1 is its own successor, by definition dominates itself,
        // but not strictly (because it equals itself), so it fits the definition of
        // the blocks in its own Dominance Frontier. But its dominance does not end
        // there, so we don't consider it part of the DF.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            jmp b1()
          b1():
            jmpif v0 then: b1, else: b2
          b2():
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main();

        let cfg = ControlFlowGraph::with_function(main);
        let post_order = PostOrder::with_cfg(&cfg);

        let mut dt = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);
        let dom_frontiers = dt.compute_dominance_frontiers(&cfg);

        assert!(dom_frontiers.is_empty());
    }

    #[test]
    fn post_dom_frontiers() {
        let ssa = loop_with_cond();
        let main = ssa.main();

        let cfg = ControlFlowGraph::with_function(main);
        let reversed_cfg = cfg.reverse();
        let post_order = PostOrder::with_cfg(&reversed_cfg);

        let mut post_dom = DominatorTree::with_cfg_and_post_order(&reversed_cfg, &post_order);
        let post_dom_frontiers = post_dom.compute_dominance_frontiers(&reversed_cfg);

        let blocks = vecmap(0..6, Id::<BasicBlock>::test_new);

        // Another way to think about the post-dominator frontier (PDF) for a node n,
        // is that we can reach a block in the PDF during execution without going through n.

        // b0 is the entry node of the program and the exit block of the post-dominator tree.
        // Thus, it has an empty set for its PDF
        assert!(!post_dom_frontiers.contains_key(&blocks[0]));
        // We must go through b1 and b2 to reach the exit node
        assert!(!post_dom_frontiers.contains_key(&blocks[1]));
        assert!(!post_dom_frontiers.contains_key(&blocks[2]));

        // b3 is the exit block of the program, but the starting node of the post-dominator tree
        // Thus, it has an empty PDF
        assert!(!post_dom_frontiers.contains_key(&blocks[3]));

        // b4 has DF { b2 } because b2 post-dominates itself and is a predecessor to b4.
        // b2 does not strictly post-dominate b4.
        let b4_pdf = &post_dom_frontiers[&blocks[4]];
        assert_eq!(b4_pdf.len(), 1);
        assert!(b4_pdf.contains(&blocks[2]));

        // Must go through b5 to reach the exit node
        assert!(!post_dom_frontiers.contains_key(&blocks[5]));
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
