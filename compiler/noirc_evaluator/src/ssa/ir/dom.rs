//! The dominator tree of a function, represented as a hash map of each reachable block id to its
//! immediate dominator.
//!
//! Dominator trees are useful for tasks such as identifying back-edges in loop analysis or
//! calculating dominance frontiers.

use std::cmp::Ordering;

#[cfg(test)]
use super::function::Function;
use super::{basic_block::BasicBlockId, cfg::ControlFlowGraph, post_order::PostOrder};
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

/// The entry and exit time of a block in a depth-first walk of the dominator tree.
///
/// Together they bracket the block's whole dominator subtree, which is what makes
/// [`DominatorTree::dominates`] a pair of integer comparisons: `a` dominates `b` exactly when
/// `b`'s interval nests inside `a`'s.
#[derive(Clone, Copy, PartialEq, Eq)]
struct DfsInterval {
    entry: u32,
    exit: u32,
}

impl DfsInterval {
    /// The interval of a block that has no place in the dominator tree, either because it is
    /// unreachable or because the tree was built without dominance queries.
    ///
    /// It is empty rather than merely out of range, so no pair of real intervals can be confused
    /// with it, and [`DominatorTree::dfs_interval`] rejects it before any comparison is made.
    const ABSENT: Self = Self { entry: u32::MAX, exit: 0 };

    /// Does this interval nest `other` inside itself?
    fn contains(self, other: Self) -> bool {
        self.entry <= other.entry && other.exit <= self.exit
    }
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

    /// The depth-first interval of each block, indexed by `BasicBlockId::to_u32`.
    ///
    /// Block ids are dense, so a flat array keeps `dominates` down to two loads and two integer
    /// comparisons; reaching the intervals through `nodes` would put a hash lookup on either side
    /// of that. Blocks with no place in the tree hold [`DfsInterval::ABSENT`], and the array is
    /// empty when the tree was built with [`DominanceQueries::Disabled`].
    dfs_intervals: Vec<DfsInterval>,
}

/// Whether a [`DominatorTree`] should be able to answer [`DominatorTree::dominates`].
///
/// The intervals that make that query cheap are a linear pass over the tree at construction, and
/// several callers only ever ask for [`DominatorTree::immediate_dominator`] or
/// [`DominatorTree::common_dominator`], which the tree answers without them.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DominanceQueries {
    Enabled,
    Disabled,
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
        let node = self.nodes.get(&block_id)?;
        node.immediate_dominator
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
    pub(crate) fn dominates(&self, block_a_id: BasicBlockId, block_b_id: BasicBlockId) -> bool {
        // `a` dominates `b` exactly when `b` sits in `a`'s subtree of the dominator tree, and a
        // depth-first walk brackets each subtree in a contiguous interval, so subtree membership
        // is interval containment. Reflexive because a block's own interval contains itself.
        self.dfs_interval(block_a_id).contains(self.dfs_interval(block_b_id))
    }

    fn dfs_interval(&self, block_id: BasicBlockId) -> DfsInterval {
        match self.dfs_intervals.get(block_id.to_u32() as usize) {
            Some(&interval) if interval != DfsInterval::ABSENT => interval,
            _ => self.no_dfs_interval(block_id),
        }
    }

    /// Panics with whichever of the two reasons a block can have no interval applies here.
    #[cold]
    #[inline(never)]
    fn no_dfs_interval(&self, block_id: BasicBlockId) -> ! {
        assert!(
            !self.dfs_intervals.is_empty() || self.nodes.is_empty(),
            "`dominates` needs a dominator tree built with `DominanceQueries::Enabled`"
        );
        panic!("Dominance for unreachable block {block_id} is undefined");
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
    pub(crate) fn with_cfg_and_post_order(
        cfg: &ControlFlowGraph,
        post_order: &PostOrder,
        queries: DominanceQueries,
    ) -> Self {
        let mut dom_tree = DominatorTree::default();
        dom_tree.compute_dominator_tree(cfg, post_order, queries);
        dom_tree
    }

    /// Allocate and compute a dominator tree for the given function.
    ///
    /// This approach computes the control flow graph and post-order internally and then
    /// discards them. If either should be retained reuse it is better to instead pre-compute them
    /// and build the dominator tree with `DominatorTree::with_cfg_and_post_order`.
    #[cfg(test)]
    pub(crate) fn with_function(func: &Function) -> Self {
        let cfg = ControlFlowGraph::with_function(func);
        let post_order = PostOrder::with_cfg(&cfg);
        Self::with_cfg_and_post_order(&cfg, &post_order, DominanceQueries::Enabled)
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
        Self::with_cfg_and_post_order(&reversed_cfg, &post_order, DominanceQueries::Enabled)
    }

    /// Build a dominator tree from a control flow graph using Keith D. Cooper's
    /// "Simple, Fast Dominator Algorithm."
    fn compute_dominator_tree(
        &mut self,
        cfg: &ControlFlowGraph,
        post_order: &PostOrder,
        queries: DominanceQueries,
    ) {
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
                changed |= self
                    .nodes
                    .get_mut(&block_id)
                    .expect("Assigned in first pass")
                    .update_estimate(immediate_dominator);
            }
        }

        if queries == DominanceQueries::Enabled {
            self.compute_dfs_intervals(*entry_block_id);
        }
    }

    /// Number every node with the entry and exit time of a depth-first walk of the dominator tree,
    /// so that dominance is interval containment rather than a walk up the tree.
    ///
    /// The walk is never actually performed. A block's immediate dominator always precedes it in
    /// the reverse post-order, so one descending pass over that order accumulates subtree sizes
    /// into parents, and one ascending pass hands each node the next free slot in its parent's
    /// interval — which is the depth-first numbering, in two linear scans of a flat array.
    fn compute_dfs_intervals(&mut self, entry_block: BasicBlockId) {
        let num_nodes = self.nodes.len();
        debug_assert_eq!(self.reverse_post_order_idx(entry_block), Some(0));

        // Most functions are a single block, which is its own whole subtree. Saying so here keeps
        // the scratch buffers below off the common path entirely.
        if num_nodes == 1 {
            self.dfs_intervals = vec![DfsInterval::ABSENT; entry_block.to_u32() as usize + 1];
            self.dfs_intervals[entry_block.to_u32() as usize] = DfsInterval { entry: 0, exit: 0 };
            return;
        }

        // The blocks in reverse post-order, and the reverse post-order index of each one's
        // immediate dominator. The entry block's parent slot is unused.
        let mut blocks = vec![entry_block; num_nodes];
        let mut parents = vec![0u32; num_nodes];
        for (&block_id, node) in &self.nodes {
            let idx = node.reverse_post_order_idx as usize;
            blocks[idx] = block_id;
            if let Some(immediate_dominator) = node.immediate_dominator {
                parents[idx] = self
                    .reverse_post_order_idx(immediate_dominator)
                    .expect("Immediate dominator is a reachable block");
            }
        }
        debug_assert_eq!(
            blocks.iter().collect::<HashSet<_>>().len(),
            num_nodes,
            "reverse post-order indices must be a permutation of 0..n"
        );

        // Children precede their parents here, so every subtree is complete before it is counted
        // into the subtree above it.
        let mut subtree_sizes = vec![1u32; num_nodes];
        for idx in (1..num_nodes).rev() {
            let parent = parents[idx] as usize;
            debug_assert!(
                parent < idx,
                "a block's immediate dominator must precede it in the reverse post-order: both \
                 scans below read their parents' slots before writing their own, so a parent that \
                 sorted after its child silently corrupts the numbering for the whole function"
            );
            subtree_sizes[parent] += subtree_sizes[idx];
        }

        // And parents precede their children here, so each node's interval is already open when
        // its children come to claim slots inside it.
        let capacity = blocks.iter().map(|block| block.to_u32() as usize + 1).max().unwrap_or(0);
        self.dfs_intervals = vec![DfsInterval::ABSENT; capacity];
        let mut next_free_slot = vec![0u32; num_nodes];
        next_free_slot[0] = 1;
        self.dfs_intervals[entry_block.to_u32() as usize] =
            DfsInterval { entry: 0, exit: num_nodes as u32 - 1 };
        for idx in 1..num_nodes {
            let parent = parents[idx] as usize;
            let entry = next_free_slot[parent];
            next_free_slot[parent] += subtree_sizes[idx];
            next_free_slot[idx] = entry + 1;
            self.dfs_intervals[blocks[idx].to_u32() as usize] =
                DfsInterval { entry, exit: entry + subtree_sizes[idx] - 1 };
        }

        // Every node handed out exactly as many slots as its subtree has members, so the
        // intervals nest and the root's spans them all. This catches any slip in the arithmetic
        // above, where a wrong subtree size stays a perfectly plausible-looking interval.
        debug_assert!((0..num_nodes).all(|idx| {
            let interval = self.dfs_intervals[blocks[idx].to_u32() as usize];
            next_free_slot[idx] == interval.entry + subtree_sizes[idx]
                && interval.exit == interval.entry + subtree_sizes[idx] - 1
        }));
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

        assert_eq!(block_a_id, block_b_id, "Unreachable block passed to common_dominator?");
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
    ///
    /// Note: this variant filters out back-edges, so loop headers are NOT included in the
    /// frontier of loop body blocks. Use `compute_dominance_frontiers_with_back_edges` for
    /// the standard definition needed by SSA construction (block parameter placement).
    pub(crate) fn compute_dominance_frontiers(
        &self,
        cfg: &ControlFlowGraph,
    ) -> HashMap<BasicBlockId, HashSet<BasicBlockId>> {
        self.compute_dominance_frontiers_inner(cfg, false)
    }

    /// Compute dominance frontiers using the standard definition (Cytron et al. 1991).
    ///
    /// Unlike `compute_dominance_frontiers`, this includes loop headers in the frontier of
    /// loop body blocks. This matches the standard definition: DF(X) = { Y | ∃ pred Z of Y:
    /// X dom Z ∧ X !sdom Y }. The standard definition is required for correct block parameter
    /// placement during SSA construction (e.g., in mem2reg).
    pub(crate) fn compute_dominance_frontiers_with_back_edges(
        &self,
        cfg: &ControlFlowGraph,
    ) -> HashMap<BasicBlockId, HashSet<BasicBlockId>> {
        self.compute_dominance_frontiers_inner(cfg, true)
    }

    fn compute_dominance_frontiers_inner(
        &self,
        cfg: &ControlFlowGraph,
        include_back_edges: bool,
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
                    //
                    // When `include_back_edges` is true (standard SSA definition), we skip this check
                    // so loop headers ARE included in the frontier of loop body blocks. This is needed
                    // for correct block parameter placement in mem2reg.
                    if !include_back_edges && self.dominates(block_id, runner) {
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
            dom::{DominanceQueries, DominatorTree},
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
        let dom_tree = DominatorTree::with_function(&func);
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

        builder.terminate_with_jmpif_no_args(cond, block2_id, block3_id);
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
        let (dt, b0, _b1, b2, b3) = unreachable_node_setup();

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
        dom_tree: DominatorTree,
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
        let dt = DominatorTree::with_function(&func);

        let cfg = ControlFlowGraph::with_function(&func);
        let dom_frontiers = dt.compute_dominance_frontiers(&cfg);
        assert!(dom_frontiers.is_empty());
    }

    #[test]
    fn post_dom_frontiers_backwards_layout() {
        let func = backwards_layout_setup();
        let post_dom = DominatorTree::with_function_post_dom(&func);

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
            jmpif v8 then: b2(), else: b3()
          b2():
            jmpif v5 then: b4(), else: b5()
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

        let post_dom = DominatorTree::with_cfg_and_post_order(
            &reversed_cfg,
            &post_order,
            DominanceQueries::Enabled,
        );

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

        let dt =
            DominatorTree::with_cfg_and_post_order(&cfg, &post_order, DominanceQueries::Enabled);
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
            jmpif v0 then: b1(), else: b2()
          b2():
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main();

        let cfg = ControlFlowGraph::with_function(main);
        let post_order = PostOrder::with_cfg(&cfg);

        let dt =
            DominatorTree::with_cfg_and_post_order(&cfg, &post_order, DominanceQueries::Enabled);
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

        let post_dom = DominatorTree::with_cfg_and_post_order(
            &reversed_cfg,
            &post_order,
            DominanceQueries::Enabled,
        );
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

/// Differential and stress tests for the dominance query, kept separate from the hand-written
/// cases above: these build randomised control flow graphs and check every dominance answer
/// against dominator sets computed by a straightforward iterative dataflow fixpoint.
#[cfg(test)]
mod differential_tests {
    use std::cmp::Ordering;

    use rustc_hash::FxHashMap as HashMap;

    use super::{DominanceQueries, DominatorTree};
    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            basic_block::BasicBlockId, cfg::ControlFlowGraph, function::Function, map::Id,
            post_order::PostOrder, types::NumericType,
        },
        ssa_gen::Ssa,
    };

    /// Deterministic xorshift so a failure reproduces from its seed alone.
    struct Rng(u64);

    impl Rng {
        fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }

        fn below(&mut self, n: usize) -> usize {
            (self.next_u64() % n as u64) as usize
        }
    }

    /// Build a function whose control flow graph is arbitrary: any block may jump to any block,
    /// which produces self loops, irreducible loops, multiple exits and unreachable blocks.
    fn random_function(seed: u64, num_blocks: usize) -> Ssa {
        let mut rng = Rng(seed);
        let mut builder = FunctionBuilder::new("func".into(), Id::test_new(0));

        let mut blocks = vec![builder.current_block()];
        for _ in 1..num_blocks {
            blocks.push(builder.insert_block());
        }

        // Targets are drawn from the non-entry blocks: a jump back to the entry leaves the
        // post-order empty and the dominator tree unpopulated, which tests nothing.
        let targets = &blocks[1..];
        let condition = builder.numeric_constant(1u128, NumericType::bool());
        for &block in &blocks {
            builder.switch_to_block(block);
            match rng.below(10) {
                0..=1 => builder.terminate_with_return(vec![]),
                2..=4 => {
                    let target = targets[rng.below(targets.len())];
                    builder.terminate_with_jmp(target, vec![]);
                }
                _ => {
                    let then_target = targets[rng.below(targets.len())];
                    let else_target = targets[rng.below(targets.len())];
                    builder.terminate_with_jmpif_no_args(condition, then_target, else_target);
                }
            }
        }

        builder.finish()
    }

    /// Dominator sets by iterative dataflow: `dom(entry) = {entry}` and
    /// `dom(b) = {b} + intersection of dom(p) over p in preds(b)`, iterated to a fixpoint.
    ///
    /// Restricted to the node set the dominator tree itself covers (the post-order), so that both
    /// sides are answering the same question on the same graph.
    fn reference_dominator_sets(
        cfg: &ControlFlowGraph,
        post_order: &[BasicBlockId],
    ) -> HashMap<BasicBlockId, Vec<bool>> {
        let index_of: HashMap<BasicBlockId, usize> =
            post_order.iter().enumerate().map(|(i, &b)| (b, i)).collect();
        let num_nodes = post_order.len();
        let entry = *post_order.last().expect("non-empty post-order");

        let mut dominators: HashMap<BasicBlockId, Vec<bool>> =
            post_order.iter().map(|&b| (b, vec![true; num_nodes])).collect();
        let mut entry_set = vec![false; num_nodes];
        entry_set[index_of[&entry]] = true;
        dominators.insert(entry, entry_set);

        let mut changed = true;
        while changed {
            changed = false;
            // Reverse post-order, entry first, so information propagates fast.
            for &block in post_order.iter().rev() {
                if block == entry {
                    continue;
                }
                let mut new_set: Option<Vec<bool>> = None;
                for predecessor in cfg.predecessors(block) {
                    let Some(predecessor_set) = dominators.get(&predecessor) else {
                        continue; // Not covered by the post-order.
                    };
                    match &mut new_set {
                        None => new_set = Some(predecessor_set.clone()),
                        Some(set) => {
                            for (slot, dominated) in set.iter_mut().zip(predecessor_set) {
                                *slot &= *dominated;
                            }
                        }
                    }
                }
                let mut new_set = new_set.unwrap_or_else(|| vec![false; num_nodes]);
                new_set[index_of[&block]] = true;
                if dominators[&block] != new_set {
                    dominators.insert(block, new_set);
                    changed = true;
                }
            }
        }

        dominators
    }

    /// The dominance query as it was before the depth-first interval numbering: walk up the
    /// dominator tree from `b` until we meet or pass `a` in the reverse post-order.
    fn dominates_by_walking(
        tree: &DominatorTree,
        block_a_id: BasicBlockId,
        mut block_b_id: BasicBlockId,
    ) -> bool {
        loop {
            match tree.reverse_post_order_cmp(block_a_id, block_b_id) {
                Ordering::Less => {
                    block_b_id = match tree.immediate_dominator(block_b_id) {
                        Some(immediate_dominator) => immediate_dominator,
                        None => return false,
                    }
                }
                Ordering::Greater => return false,
                Ordering::Equal => return true,
            }
        }
    }

    fn check_tree_against_reference(
        tree: &DominatorTree,
        cfg: &ControlFlowGraph,
        post_order: &[BasicBlockId],
        context: &str,
    ) {
        let index_of: HashMap<BasicBlockId, usize> =
            post_order.iter().enumerate().map(|(i, &b)| (b, i)).collect();
        let dominators = reference_dominator_sets(cfg, post_order);

        for &b in post_order {
            for &a in post_order {
                let expected = dominators[&b][index_of[&a]];
                assert_eq!(
                    tree.dominates(a, b),
                    expected,
                    "{context}: dominates({a}, {b}) disagrees with the reference dominator sets"
                );
                assert_eq!(
                    dominates_by_walking(tree, a, b),
                    expected,
                    "{context}: the reference dominator sets disagree with the tree walk for \
                     ({a}, {b}), so the reference itself is suspect"
                );
            }

            // The immediate dominator is the strict dominator dominated by every other strict
            // dominator of the block.
            let strict_dominators: Vec<_> = post_order
                .iter()
                .copied()
                .filter(|&a| a != b && dominators[&b][index_of[&a]])
                .collect();
            let expected_immediate = strict_dominators
                .iter()
                .copied()
                .find(|&a| strict_dominators.iter().all(|&other| dominators[&a][index_of[&other]]));
            assert_eq!(
                tree.immediate_dominator(b),
                expected_immediate,
                "{context}: immediate_dominator({b}) is wrong"
            );
        }
    }

    #[test]
    fn random_control_flow_graphs_agree_with_reference_dominator_sets() {
        let mut checked = 0usize;
        let mut skipped = 0usize;
        let mut total_reachable = 0usize;
        let mut max_reachable = 0usize;
        let mut with_irreducible = 0usize;
        let mut pairs = 0usize;
        for seed in 1..2000u64 {
            for num_blocks in [2usize, 3, 5, 8, 13, 21, 34] {
                let ssa = random_function(seed, num_blocks);
                let func = ssa.main();
                let cfg = ControlFlowGraph::with_function(func);
                let post_order = PostOrder::with_cfg(&cfg);
                if post_order.as_slice().is_empty() {
                    skipped += 1;
                    continue;
                }
                let tree = DominatorTree::with_cfg_and_post_order(
                    &cfg,
                    &post_order,
                    DominanceQueries::Enabled,
                );
                check_tree_against_reference(
                    &tree,
                    &cfg,
                    post_order.as_slice(),
                    &format!("seed {seed}, {num_blocks} blocks"),
                );
                let n = post_order.as_slice().len();
                checked += 1;
                total_reachable += n;
                max_reachable = max_reachable.max(n);
                pairs += n * n;
                if is_irreducible(&cfg, post_order.as_slice()) {
                    with_irreducible += 1;
                }
            }
        }
        assert!(checked > 10_000, "coverage too thin: only {checked} graphs checked");
        assert!(with_irreducible > 1000, "no irreducible control flow exercised");
    }

    /// A CFG is irreducible when some loop has more than one entry, which shows up as a
    /// back edge whose target does not dominate its source.
    fn is_irreducible(cfg: &ControlFlowGraph, post_order: &[BasicBlockId]) -> bool {
        let index_of: HashMap<BasicBlockId, usize> =
            post_order.iter().rev().enumerate().map(|(i, &b)| (b, i)).collect();
        let dominators = reference_dominator_sets(cfg, post_order);
        let forward_index: HashMap<BasicBlockId, usize> =
            post_order.iter().enumerate().map(|(i, &b)| (b, i)).collect();
        for (&block, &block_index) in &index_of {
            for predecessor in cfg.predecessors(block) {
                let Some(&predecessor_index) = index_of.get(&predecessor) else { continue };
                // A back edge in the reverse post-order whose target does not dominate its source.
                if predecessor_index >= block_index
                    && !dominators[&predecessor][forward_index[&block]]
                {
                    return true;
                }
            }
        }
        false
    }

    /// The other construction path in the tree: the post-dominator tree that `loop_invariant`
    /// builds over the extended reversed CFG, whose root is the (possibly synthesised) exit node.
    #[test]
    fn random_post_dominator_trees_agree_with_reference_dominator_sets() {
        let mut checked = 0usize;
        let mut synthesised_exit = 0usize;
        for seed in 1..2000u64 {
            for num_blocks in [2usize, 3, 5, 8, 13, 21] {
                let mut ssa = random_function(seed, num_blocks);
                let func = ssa.main_mut();
                let (would_ice, needs_exit) = extended_reverse_shape(func);
                if would_ice {
                    continue;
                }
                if needs_exit {
                    synthesised_exit += 1;
                }
                let reversed_cfg = ControlFlowGraph::extended_reverse(func);
                let post_order = PostOrder::with_cfg(&reversed_cfg);
                if post_order.as_slice().is_empty() {
                    continue;
                }
                let tree = DominatorTree::with_cfg_and_post_order(
                    &reversed_cfg,
                    &post_order,
                    DominanceQueries::Enabled,
                );
                check_tree_against_reference(
                    &tree,
                    &reversed_cfg,
                    post_order.as_slice(),
                    &format!("post-dom seed {seed}, {num_blocks} blocks"),
                );
                checked += 1;
            }
        }
        assert!(checked > 9000, "coverage too thin: only {checked} post-dominator trees checked");
        assert!(synthesised_exit > 1000, "the synthesised exit node was barely exercised");
    }

    /// `ControlFlowGraph::extended_reverse` wires every block that cannot reach an exit into a
    /// synthesised exit node, and asserts on the way that no block gains a third successor. A
    /// block sitting in an infinite loop behind a conditional branch already has two, so these
    /// graphs are excluded here rather than tripping an assertion unrelated to dominance.
    fn extended_reverse_shape(func: &Function) -> (bool, bool) {
        let cfg = ControlFlowGraph::with_function(func);
        let exits: Vec<_> = func
            .reachable_blocks()
            .into_iter()
            .filter(|&block| cfg.successors(block).len() == 0)
            .collect();
        let reverse = cfg.reverse();
        let reaches_an_exit: std::collections::HashSet<_> =
            PostOrder::with_cfg(&reverse).into_vec().into_iter().collect();
        let dead: Vec<_> = func
            .reachable_blocks()
            .into_iter()
            .filter(|block| !reaches_an_exit.contains(block))
            .collect();
        let needs_exit = exits.len() > 1 || !dead.is_empty();
        let would_ice = needs_exit && dead.iter().any(|&block| cfg.successors(block).len() >= 2);
        (would_ice, needs_exit)
    }

    #[test]
    fn dominance_is_a_partial_order_on_random_graphs() {
        for seed in 1..200u64 {
            let ssa = random_function(seed, 10);
            let func = ssa.main();
            let cfg = ControlFlowGraph::with_function(func);
            let post_order = PostOrder::with_cfg(&cfg);
            if post_order.as_slice().is_empty() {
                continue;
            }
            let tree = DominatorTree::with_cfg_and_post_order(
                &cfg,
                &post_order,
                DominanceQueries::Enabled,
            );
            let blocks = post_order.as_slice();

            for &a in blocks {
                assert!(tree.dominates(a, a), "seed {seed}: dominance is not reflexive at {a}");
                for &b in blocks {
                    if a != b && tree.dominates(a, b) {
                        assert!(
                            !tree.dominates(b, a),
                            "seed {seed}: {a} and {b} dominate each other"
                        );
                    }
                    for &c in blocks {
                        if tree.dominates(a, b) && tree.dominates(b, c) {
                            assert!(
                                tree.dominates(a, c),
                                "seed {seed}: dominance is not transitive for {a}, {b}, {c}"
                            );
                        }
                    }
                }
            }
        }
    }

    /// Every node's immediate dominator must come earlier in the reverse post-order: the
    /// two-scan interval numbering reads its parents' slots before writing its own, so a parent
    /// that sorted after its child would silently corrupt the numbering for the whole function
    /// rather than for one pair.
    #[test]
    fn immediate_dominators_precede_their_children_in_the_reverse_post_order() {
        for seed in 1..400u64 {
            for num_blocks in [2usize, 3, 5, 8, 13, 21] {
                let ssa = random_function(seed, num_blocks);
                let func = ssa.main();
                let cfg = ControlFlowGraph::with_function(func);
                let post_order = PostOrder::with_cfg(&cfg);
                if post_order.as_slice().is_empty() {
                    continue;
                }
                let tree = DominatorTree::with_cfg_and_post_order(
                    &cfg,
                    &post_order,
                    DominanceQueries::Enabled,
                );
                for &block in post_order.as_slice() {
                    let Some(immediate_dominator) = tree.immediate_dominator(block) else {
                        continue;
                    };
                    assert_eq!(
                        tree.reverse_post_order_cmp(immediate_dominator, block),
                        Ordering::Less,
                        "seed {seed}, {num_blocks} blocks: immediate dominator {immediate_dominator} \
                         of {block} does not precede it in the reverse post-order"
                    );
                }
            }
        }
    }

    /// A tree built without dominance queries still answers everything that does not need the
    /// depth-first intervals, which is what `mem2reg` builds one for.
    #[test]
    fn tree_without_dominance_queries_still_answers_immediate_dominators() {
        for seed in 1..200u64 {
            let ssa = random_function(seed, 10);
            let func = ssa.main();
            let cfg = ControlFlowGraph::with_function(func);
            let post_order = PostOrder::with_cfg(&cfg);
            let enabled = DominatorTree::with_cfg_and_post_order(
                &cfg,
                &post_order,
                DominanceQueries::Enabled,
            );
            let disabled = DominatorTree::with_cfg_and_post_order(
                &cfg,
                &post_order,
                DominanceQueries::Disabled,
            );
            for &block in post_order.as_slice() {
                assert_eq!(disabled.immediate_dominator(block), enabled.immediate_dominator(block));
                assert_eq!(
                    disabled.reverse_post_order_idx(block),
                    enabled.reverse_post_order_idx(block)
                );
                assert!(disabled.is_reachable(block));
            }
            assert_eq!(
                disabled.compute_dominance_frontiers_with_back_edges(&cfg).len(),
                enabled.compute_dominance_frontiers_with_back_edges(&cfg).len()
            );
        }
    }

    #[test]
    #[should_panic(expected = "needs a dominator tree built with `DominanceQueries::Enabled`")]
    fn dominates_on_a_tree_without_dominance_queries_panics() {
        let ssa = random_function(1, 4);
        let func = ssa.main();
        let cfg = ControlFlowGraph::with_function(func);
        let post_order = PostOrder::with_cfg(&cfg);
        let tree =
            DominatorTree::with_cfg_and_post_order(&cfg, &post_order, DominanceQueries::Disabled);
        let entry = *post_order.as_slice().last().unwrap();
        tree.dominates(entry, entry);
    }

    #[test]
    #[ignore = "benchmark, run explicitly with --ignored --nocapture --release"]
    fn bench_dominance_query() {
        use std::time::Instant;

        for num_blocks in [8usize, 32, 128, 512] {
            let ssa = random_function(12345, num_blocks);
            let func = ssa.main();
            let cfg = ControlFlowGraph::with_function(func);
            let post_order = PostOrder::with_cfg(&cfg);
            let tree = DominatorTree::with_cfg_and_post_order(
                &cfg,
                &post_order,
                DominanceQueries::Enabled,
            );
            let blocks: Vec<_> = post_order.as_slice().to_vec();
            let pairs: Vec<_> = blocks
                .iter()
                .flat_map(|&a| blocks.iter().map(move |&b| (a, b)))
                .cycle()
                .take(2_000_000)
                .collect();

            let start = Instant::now();
            let mut accumulator = 0usize;
            for &(a, b) in &pairs {
                accumulator += usize::from(tree.dominates(a, b));
            }
            let queries = start.elapsed();
            std::hint::black_box(accumulator);

            let time = |queries| {
                let start = Instant::now();
                for _ in 0..1000 {
                    std::hint::black_box(DominatorTree::with_cfg_and_post_order(
                        &cfg,
                        &post_order,
                        queries,
                    ));
                }
                start.elapsed()
            };
            let with_intervals = time(DominanceQueries::Enabled);
            let without_intervals = time(DominanceQueries::Disabled);

            println!(
                "{:>4} reachable blocks | {} queries in {:>9.3?} ({:>5.2} ns each) | 1000 \
                 constructions: {:>9.3?} with intervals, {:>9.3?} without",
                blocks.len(),
                pairs.len(),
                queries,
                queries.as_nanos() as f64 / pairs.len() as f64,
                with_intervals,
                without_intervals,
            );
        }
    }
}
