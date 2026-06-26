//! The post-order for a given function represented as a vector of basic block ids.
//!
//! This ordering is beneficial to the efficiency of various algorithms, such as those for dead
//! code elimination and calculating dominance trees.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use rustc_hash::FxHashMap as HashMap;

use crate::ssa::ir::{basic_block::BasicBlockId, function::Function};

use super::cfg::ControlFlowGraph;

/// Depth-first traversal stack state marker for computing the cfg post-order.
enum Visit {
    First,
    Last,
}

/// In the post-order, each block is visited after all of its successors.
#[derive(Default, Clone)]
pub(crate) struct PostOrder(Vec<BasicBlockId>);

impl PostOrder {
    pub(crate) fn as_slice(&self) -> &[BasicBlockId] {
        self.0.as_slice()
    }
}

impl PostOrder {
    /// Allocate and compute a function's block post-order.
    pub(crate) fn with_function(func: &Function) -> Self {
        let cfg = ControlFlowGraph::with_function(func);
        Self::with_cfg(&cfg)
    }

    /// Allocate and compute a function's block post-order.
    pub(crate) fn with_cfg(cfg: &ControlFlowGraph) -> Self {
        let roots = cfg.compute_entry_blocks();
        PostOrder(Self::compute_post_order(cfg, roots))
    }

    /// Allocate and compute a function's block post-order, always rooted at the
    /// function's entry block.
    ///
    /// Unlike [`Self::with_function`], this includes the entry block even when
    /// it has incoming edges (e.g. malformed SSA with a back-edge to entry).
    pub(crate) fn with_function_from_entry(func: &Function) -> Self {
        let cfg = ControlFlowGraph::with_function(func);
        PostOrder(Self::compute_post_order(&cfg, vec![func.entry_block()]))
    }

    /// Return blocks in post-order.
    pub(crate) fn into_vec(self) -> Vec<BasicBlockId> {
        self.0
    }

    /// Return blocks in reverse-post-order (RPO).
    ///
    /// In RPO, each block is visited only after all of its predecessors, except for the
    /// back-edge predecessors of loop headers. Blocks are ordered by the maximum cost to reach
    /// them from the entry block (the longest path once back-edges are removed), with ties
    /// broken by block id. The cost of each loop's exit blocks is then raised above the loop
    /// body so that every loop's blocks are contiguous and precede the blocks reached by
    /// exiting the loop.
    ///
    /// Take this CFG for example:
    /// ```text
    ///      b0
    ///      |
    ///      b1<-+
    ///     /  \ |
    ///    b3   b2
    /// ```
    /// The RPO is `[b0, b1, b2, b3]`: the loop body `b2` is ordered before the loop exit `b3`.
    pub(crate) fn into_vec_reverse(self) -> Vec<BasicBlockId> {
        let mut blocks = self.into_vec();
        blocks.reverse();
        blocks
    }

    /// Computes the post-order of the CFG so that, in the reverse-post-order, each block is
    /// visited only after all of its non-back-edge predecessors.
    ///
    /// A plain depth-first post-order already has this property for back-edge-free CFGs, but it
    /// can order a loop exit ahead of the loop body (see
    /// <https://github.com/noir-lang/noir/issues/9771>). To get a deterministic, well-behaved
    /// order we rank each block by the *maximum cost to reach it* from a root: the length of the
    /// longest path once back-edges are removed. Sorting by `(cost, block id)` is always a valid
    /// topological order of the back-edge-free CFG, since any forward edge `u -> v` forces
    /// `cost(v) >= cost(u) + 1`.
    ///
    /// Each block is visited at most once during the depth-first traversal; the cost of each
    /// block is then computed in a single linear scan before a final sort.
    ///
    /// After the base costs are computed, the cost of each loop's exit blocks is raised above
    /// the loop body (see [`Self::raise_loop_exit_costs`]) so that loops stay contiguous and
    /// the blocks reached by exiting a loop are ordered after the entire loop body.
    fn compute_post_order(cfg: &ControlFlowGraph, roots: Vec<BasicBlockId>) -> Vec<BasicBlockId> {
        let post_order = Self::depth_first_post_order(cfg, roots);

        // A block's index in the depth-first reverse-post-order. Crucially, every non-back edge
        // points forwards in this order (smaller index -> larger index) while every back edge
        // points backwards. So a predecessor `u` of `v` is a non-back-edge ("forward")
        // predecessor exactly when `rpo_index[u] < rpo_index[v]`; no explicit edge classification
        // is needed.
        let rpo_index: HashMap<BasicBlockId, u32> = post_order
            .iter()
            .rev()
            .enumerate()
            .map(|(index, block)| (*block, index as u32))
            .collect();

        // The maximum cost to reach each block: `cost(v) = max(cost(u) + 1)` over the forward
        // predecessors `u` of `v`, or 0 for a root. Iterating in reverse-post-order guarantees
        // every forward predecessor has already been assigned its cost. Predecessors that are
        // unreachable from the roots have no `rpo_index` and are ignored, as are back-edge
        // predecessors (which have a larger `rpo_index`).
        let mut cost: HashMap<BasicBlockId, u32> = HashMap::default();
        for &block in post_order.iter().rev() {
            let block_index = rpo_index[&block];
            let block_cost = cfg
                .predecessors(block)
                .filter_map(|predecessor| {
                    let predecessor_index = *rpo_index.get(&predecessor)?;
                    (predecessor_index < block_index).then(|| cost[&predecessor] + 1)
                })
                .max()
                .unwrap_or(0);
            cost.insert(block, block_cost);
        }

        Self::raise_loop_exit_costs(cfg, &post_order, &rpo_index, &mut cost);

        // Order by ascending cost (ties broken by block id) to get the reverse-post-order, then
        // reverse it to recover the post-order.
        let mut reverse_post_order = post_order;
        reverse_post_order.reverse();
        reverse_post_order.sort_by_key(|block| (cost[block], *block));
        reverse_post_order.reverse();
        reverse_post_order
    }

    /// Raise the cost of every loop's exit blocks above the maximum cost within the loop,
    /// propagating each increase forward. After this, sorting by `(cost, block id)` keeps each
    /// natural loop's blocks contiguous and orders the blocks reached by exiting a loop after the
    /// entire loop body. Without it a loop exit branches off the loop header (a low-cost block)
    /// and so inherits a low cost, letting it sort ahead of deeper loop-body blocks (see
    /// <https://github.com/noir-lang/noir/issues/9771>).
    ///
    /// An exit is a successor `e` of a loop block `b` that is not itself in the loop and is
    /// reached by a *forward* edge (`rpo_index[e] > rpo_index[b]`); a successor reached by a
    /// back-edge is an enclosing loop header, which legitimately precedes the loop. Note a forward
    /// exit may still have a smaller `rpo_index` than other blocks of the loop, which is why the
    /// raise is propagated rather than folded into the single cost scan.
    ///
    /// Bumping one loop's exit can raise a block of another loop (e.g. two sequential loops where
    /// the first's exit feeds the second's body), so the bumps are repeated to a fixpoint. Each
    /// raise only ever increases a cost and only propagates along forward edges, so costs are
    /// bounded by the longest forward path and the fixpoint is reached quickly; the iteration cap
    /// is a safety bound for irreducible CFGs. The result is a valid reverse-post-order regardless
    /// (every non-back edge `u -> v` keeps `cost[u] < cost[v]`), so loop-set approximation only
    /// affects contiguity, never correctness.
    fn raise_loop_exit_costs(
        cfg: &ControlFlowGraph,
        post_order: &[BasicBlockId],
        rpo_index: &HashMap<BasicBlockId, u32>,
        cost: &mut HashMap<BasicBlockId, u32>,
    ) {
        let loops = Self::collect_loop_block_sets(cfg, post_order, rpo_index);
        if loops.is_empty() {
            return;
        }

        // The forward exits of each loop are structural, so compute them once. `BTreeSet` keeps
        // the iteration order below deterministic.
        let loop_exits: Vec<BTreeSet<BasicBlockId>> = loops
            .iter()
            .map(|blocks| {
                let mut exits = BTreeSet::new();
                for &block in blocks {
                    let block_index = rpo_index[&block];
                    for successor in cfg.successors(block) {
                        if blocks.contains(&successor) {
                            continue;
                        }
                        if rpo_index.get(&successor).is_some_and(|&index| index > block_index) {
                            exits.insert(successor);
                        }
                    }
                }
                exits
            })
            .collect();

        for _ in 0..=post_order.len() {
            let mut changed = false;
            for (blocks, exits) in loops.iter().zip(&loop_exits) {
                let Some(max_cost_in_loop) =
                    blocks.iter().filter_map(|block| cost.get(block).copied()).max()
                else {
                    continue;
                };
                for &exit in exits {
                    if cost.get(&exit).is_some_and(|&current| current < max_cost_in_loop + 1) {
                        Self::raise_cost(cfg, rpo_index, cost, exit, max_cost_in_loop + 1);
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }

    /// Raise `cost[start]` to `new_cost` and propagate the increase forward so that every forward
    /// (non-back) edge `u -> v` keeps `cost[v] >= cost[u] + 1`. Propagation only follows forward
    /// edges (strictly increasing `rpo_index`) and costs only increase, so the traversal is over a
    /// DAG and always terminates.
    fn raise_cost(
        cfg: &ControlFlowGraph,
        rpo_index: &HashMap<BasicBlockId, u32>,
        cost: &mut HashMap<BasicBlockId, u32>,
        start: BasicBlockId,
        new_cost: u32,
    ) {
        match cost.get(&start) {
            Some(&current) if current >= new_cost => return,
            None => return,
            _ => {}
        }
        cost.insert(start, new_cost);

        let mut stack = vec![start];
        while let Some(block) = stack.pop() {
            let block_cost = cost[&block];
            let Some(&block_index) = rpo_index.get(&block) else {
                continue;
            };
            for successor in cfg.successors(block) {
                let Some(&successor_index) = rpo_index.get(&successor) else {
                    continue;
                };
                if block_index < successor_index && cost[&successor] < block_cost + 1 {
                    cost.insert(successor, block_cost + 1);
                    stack.push(successor);
                }
            }
        }
    }

    /// The natural loop block sets of the CFG, one per loop header.
    ///
    /// A retreating edge `p -> h` (`rpo_index[p] >= rpo_index[h]`) is a back-edge to header `h`.
    /// The blocks of `h`'s natural loop are `h` together with every block that can reach a
    /// back-edge source without passing through `h`. When a header has several back-edges (e.g. a
    /// loop with `break`/`continue` arms that each jump back) the natural loop is the *union* of
    /// the per-edge sets — otherwise a block internal to the larger arm would look like an exit of
    /// the smaller one, and the two would fight over its position.
    fn collect_loop_block_sets(
        cfg: &ControlFlowGraph,
        post_order: &[BasicBlockId],
        rpo_index: &HashMap<BasicBlockId, u32>,
    ) -> Vec<BTreeSet<BasicBlockId>> {
        // `BTreeMap`/`BTreeSet` keep header discovery and the merged sets deterministic.
        let mut loops: BTreeMap<BasicBlockId, BTreeSet<BasicBlockId>> = BTreeMap::new();
        for &header in post_order {
            let header_index = rpo_index[&header];
            for predecessor in cfg.predecessors(header) {
                let Some(&predecessor_index) = rpo_index.get(&predecessor) else {
                    continue;
                };
                if predecessor_index >= header_index {
                    let blocks = Self::find_blocks_in_loop(cfg, header, predecessor);
                    loops.entry(header).or_default().extend(blocks);
                }
            }
        }
        loops.into_values().collect()
    }

    /// The blocks of the natural loop with the given `header` and back-edge source
    /// `back_edge_start`: collected by walking predecessors backwards from `back_edge_start`,
    /// stopping at the header.
    fn find_blocks_in_loop(
        cfg: &ControlFlowGraph,
        header: BasicBlockId,
        back_edge_start: BasicBlockId,
    ) -> BTreeSet<BasicBlockId> {
        let mut blocks = BTreeSet::new();
        blocks.insert(header);

        let mut stack = vec![back_edge_start];
        while let Some(block) = stack.pop() {
            // The header is already inserted, so reaching it returns `false` and we stop walking
            // past it; this also terminates the walk for a single-block (self-edge) loop.
            if blocks.insert(block) {
                stack.extend(cfg.predecessors(block));
            }
        }
        blocks
    }

    // Computes the post-order of the CFG by doing a depth-first traversal of the given
    // root blocks' previously unvisited children. Each block is sequenced according
    // to when the traversal exits it.
    fn depth_first_post_order(
        cfg: &ControlFlowGraph,
        roots: Vec<BasicBlockId>,
    ) -> Vec<BasicBlockId> {
        let mut stack = vec![];
        let mut visited: HashSet<BasicBlockId> = HashSet::new();
        let mut post_order: Vec<BasicBlockId> = Vec::new();

        // Set root blocks
        stack.extend(roots.into_iter().map(|root| (Visit::First, root)));

        while let Some((visit, block_id)) = stack.pop() {
            match visit {
                Visit::First => {
                    if !visited.contains(&block_id) {
                        // This is the first time we pop the block, so we need to scan its
                        // successors and then revisit it.
                        visited.insert(block_id);
                        stack.push((Visit::Last, block_id));
                        // Stack successors for visiting. Because items are taken from the top of the
                        // stack, we push the item that's due for a visit first to the top.
                        for successor_id in cfg.successors(block_id).rev() {
                            if !visited.contains(&successor_id) {
                                // This not visited check would also be covered by the next
                                // iteration, but checking here too saves an iteration per successor.
                                stack.push((Visit::First, successor_id));
                            }
                        }
                    }
                }

                Visit::Last => {
                    // We've finished all this node's successors.
                    post_order.push(block_id);
                }
            }
        }
        post_order
    }
}

#[cfg(test)]
mod tests {
    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            basic_block::BasicBlockId,
            function::Function,
            map::Id,
            post_order::PostOrder,
            types::{NumericType, Type},
        },
        ssa_gen::Ssa,
    };

    #[test]
    fn single_block() {
        let func_id = Id::test_new(0);
        let func = Function::new("func".into(), func_id);
        let post_order = PostOrder::with_function(&func);
        assert_eq!(post_order.0, [func.entry_block()]);
    }

    #[test]
    fn arb_graph_with_unreachable() {
        // A → B   C
        // ↓ ↗ ↓   ↓
        // D ← E → F
        // (`A` is entry block)
        // Expected post-order working:
        // A {
        //   B {
        //     E {
        //       D {
        //         B (seen)
        //       } -> push(D)
        //       F {
        //       } -> push(F)
        //     } -> push(E)
        //   } -> push(B)
        //   D (seen)
        // } -> push(A)
        // Result:
        // D, F, E, B, A, (C dropped as unreachable)

        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id);
        let block_b_id = builder.insert_block();
        let block_c_id = builder.insert_block();
        let block_d_id = builder.insert_block();
        let block_e_id = builder.insert_block();
        let block_f_id = builder.insert_block();
        let cond_a = builder.add_parameter(Type::unsigned(1));
        let cond_e = builder.add_parameter(Type::unsigned(1));

        // A → B   •
        // ↓
        // D   •   •
        builder.terminate_with_jmpif_no_args(cond_a, block_b_id, block_d_id);
        //  •   B   •
        //  •   ↓   •
        //  •   E   •
        builder.switch_to_block(block_b_id);
        builder.terminate_with_jmp(block_e_id, vec![]);
        // •   •   •
        //
        // D ← E → F
        builder.switch_to_block(block_e_id);
        builder.terminate_with_jmpif_no_args(cond_e, block_d_id, block_f_id);
        // •   B   •
        //   ↗
        // D   •   •
        builder.switch_to_block(block_d_id);
        builder.terminate_with_jmp(block_b_id, vec![]);
        // •   •   C
        // •   •   ↓
        // •   •   F
        builder.switch_to_block(block_c_id);
        builder.terminate_with_jmp(block_f_id, vec![]);

        builder.switch_to_block(block_f_id);
        builder.terminate_with_return(vec![]);

        let ssa = builder.finish();
        let func = ssa.main();
        let post_order = PostOrder::with_function(func);
        let block_a_id = func.entry_block();
        // Blocks are ordered by their maximum cost to reach them from the entry block.
        // Costs (longest path from A, ignoring the D->B back-edge): A=0, B=1, E=2, D=3, F=3.
        // The reverse-post-order is therefore [A, B, E, D, F], so the post-order is its reverse.
        assert_eq!(post_order.0, [block_f_id, block_d_id, block_e_id, block_b_id, block_a_id]);
    }

    /// Helper to construct a `BasicBlockId` with a syntax resembling the `b0`
    /// syntax used in comments/ssa output.
    fn b(id: u32) -> BasicBlockId {
        BasicBlockId::test_new(id)
    }

    /// Regression test for <https://github.com/noir-lang/noir/issues/9771>: the loop exit `b3`
    /// must be ordered after the loop body `b2` (and thus appear first in post-order).
    #[test]
    fn loop_regression() {
        // b0 -> b1 <-> b2
        //        |
        //        V
        //       b3
        let mut builder = FunctionBuilder::new("func".into(), Id::test_new(0));
        let b0 = builder.current_block();
        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        let b3 = builder.insert_block();

        // This needs to use the FunctionBuilder since the Ssa parser will change the block ids
        builder.terminate_with_jmp(b1, Vec::new());

        builder.switch_to_block(b1);
        let zero = builder.numeric_constant(0u32, NumericType::bool());
        builder.terminate_with_jmpif_no_args(zero, b2, b3);

        builder.switch_to_block(b2);
        builder.terminate_with_jmp(b1, Vec::new());

        builder.switch_to_block(b3);
        builder.terminate_with_return(Vec::new());
        let ssa = builder.finish();

        let func = ssa.main();
        let post_order = PostOrder::with_function(func);

        // Costs (longest path from b0, ignoring the b2->b1 back-edge): b0=0, b1=1, b2=2, b3=2.
        // The reverse-post-order is [b0, b1, b2, b3] (b2 before b3 by block-id tie-break), so
        // the loop exit b3 is now correctly ordered last in RPO / first in post-order.
        assert_eq!(post_order.0, [b3, b2, b1, b0]);
    }

    /// A loop whose body and back-edge live in different blocks (a separate "latch"). The
    /// loop exit branches off the header, so under a plain max-cost order it is hoisted ahead
    /// of the deeper latch block. The order must keep the whole loop body before the exit.
    #[test]
    fn loop_with_separate_latch() {
        // b0 -> b1 <--+
        //       |\    |
        //       | b2  |
        //       |  \  |
        //       |  b4-+   (b4 is the latch: jumps back to the header b1)
        //       V
        //      b3        (loop exit)
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmp b1()
          b1():
            jmpif v0 then: b2(), else: b3()
          b2():
            jmp b4()
          b3():
            return
          b4():
            jmp b1()
        }";

        let ssa = Ssa::from_str(src).unwrap();
        let func = ssa.main();
        let post_order = PostOrder::with_function(func);

        // Loop = {b1, b2, b4}, exit = b3. The exit b3 must come after the entire loop body in
        // the forward order, so it must appear *first* in the post-order:
        // forward order [b0, b1, b2, b4, b3] -> post-order [b3, b4, b2, b1, b0].
        assert_eq!(post_order.0, [b(3), b(4), b(2), b(1), b(0)]);
    }

    #[test]
    fn simple_if() {
        let src = "
        acir(inline) fn factorial f1 {
          b0(v1: u32):
            v2 = lt v1, u32 1
            jmpif v2 then: b1(), else: b2()
          b1():
            jmp b3(u32 1)
          b2():
            v4 = sub v1, u32 1
            v5 = call f1(v4) -> u32
            v6 = mul v1, v5
            jmp b3(v6)
          b3(v7: u32):
            return v7
        }";

        let ssa = Ssa::from_str(src).unwrap();
        let func = ssa.main();
        let post_order = PostOrder::with_function(func);
        // Costs (longest path from b0): b0=0, b1=1, b2=1, b3=2. b1 and b2 tie on cost, so the
        // block-id tie-break orders b1 before b2 in RPO, giving post-order [b3, b2, b1, b0].
        assert_eq!(post_order.0, [b(3), b(2), b(1), b(0)]);
    }

    #[test]
    fn nested_loop() {
        // b0 -> b1 -> b3
        //      / ^
        //     V   \
        //     b2   |
        //     |    |
        //     V    |
        //     b4->b6
        //     | ^
        //     V  \
        //     b5->b8
        //     |   ^
        //     V  /
        //     b7
        //
        // Expected topological sort:
        // [b0, {loop blocks}, b3]  (see below for more complete explanation)
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u1):
                jmp b1(v0)
              b1(v1: u1):
                jmpif v1 then: b2(), else: b3()
              b2():
                jmp b4(v1)
              b3():
                return
              b4(v2: u1):
                jmpif v2 then: b5(), else: b6()
              b5():
                jmpif v2 then: b7(), else: b8()
              b6():
                jmp b1(v1)
              b7():
                jmp b8()
              b8():
                jmp b4(v1)
            }
            ";

        // We can break the CFG above into CFG's for each loop where the start
        // node is where the loop was entered from. Break all incoming edges
        // from this node, and remove outgoing edges that lead out of the loop:
        //
        //       b1
        //      /
        //     V
        //     b2
        //     |
        //     V
        //     b4->b6
        //     | ^
        //     V  \
        //     b5->b8
        //     |   ^
        //     V  /
        //     b7
        //
        // Here the expected topological sort is more clear:
        // [ b1, b2, { loop blocks }, b6 ]
        //
        // We can do this again for the inner loop starting from `b4`:
        //
        //     b4
        //     |
        //     V
        //     b5->b8
        //     |   ^
        //     V  /
        //     b7
        //
        // Where the topological sort for this is unambiguously
        // [b4, b5, b7, b8]
        //
        // Now we can slot this in to the unknown {loop blocks} from the first loop's
        // topological sort to get:
        //
        // [b1, b2, b4, b5, b7, b8, b6]
        //
        // Blocks are first ordered by their maximum cost to reach them from the entry block,
        // i.e. the longest path from b0 once the back-edges (b6->b1 and b8->b4) are removed:
        //
        //   b0=0, b1=1, b2=2, b3=2, b4=3, b5=4, b6=4, b7=5, b8=6
        //
        // Each loop's exit costs are then raised above the loop body. The inner loop
        // {b4, b5, b7, b8} (max cost 6) pushes its exit b6 to 7; the outer loop
        // {b1, b2, b4, b5, b6, b7, b8} (now max cost 7) pushes its exit b3 to 8:
        //
        //   b0=0, b1=1, b2=2, b4=3, b5=4, b7=5, b8=6, b6=7, b3=8
        //
        // Sorting by (cost, block-id) ascending gives the reverse-post-order
        // [b0, b1, b2, b4, b5, b7, b8, b6, b3]. Every block still comes after all of its
        // (non-back-edge) predecessors, and now each loop is contiguous with its exit ordered
        // after the entire loop body. The expected post-order is the reverse:
        let expected_post_order = [b(3), b(6), b(8), b(7), b(5), b(4), b(2), b(1), b(0)];

        let ssa = Ssa::from_str(src).unwrap();

        let func = ssa.main();
        let post_order = PostOrder::with_function(func);
        assert_eq!(post_order.0, expected_post_order);
    }
}

/// Verifies, over real programs compiled through the full SSA pipeline, that the forward order
/// places every block reached by exiting a loop after the entire loop body (see
/// [`PostOrder::raise_loop_exit_costs`]).
#[cfg(test)]
mod loop_ordering_property_tests {
    use std::collections::{BTreeMap, BTreeSet};

    use rustc_hash::FxHashMap;

    use super::PostOrder;
    use crate::ssa::ir::{basic_block::BasicBlockId, cfg::ControlFlowGraph, function::Function};
    use crate::ssa::opt::{LoopOrder, Loops};
    use crate::ssa::ssa_gen::generate_ssa;
    use crate::ssa::{Ssa, SsaBuilder, SsaEvaluatorOptions, primary_passes};
    use noirc_frontend::test_utils::get_monomorphized;

    /// A place where a loop's exit block is ordered at or before a block of that loop in the
    /// function's forward (reverse-post) order. The fields are reported through `Debug` in the
    /// failure message.
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Violation {
        loop_header: BasicBlockId,
        exit_block: BasicBlockId,
        offending_loop_block: BasicBlockId,
        exit_position: usize,
        offending_position: usize,
    }

    /// For each natural loop in `func`, check that every block reached by exiting the loop comes
    /// after the entire loop body in the forward order.
    fn find_loop_exit_ordering_violations(func: &Function) -> Vec<Violation> {
        let forward = PostOrder::with_function(func).into_vec_reverse();
        let position: FxHashMap<BasicBlockId, usize> =
            forward.iter().copied().enumerate().map(|(index, block)| (block, index)).collect();

        let cfg = ControlFlowGraph::with_function(func);
        // `find_all` does not mutate the function; it only reads the CFG to discover loops. It
        // returns one entry per back-edge, so merge entries that share a header into the header's
        // natural loop (the union of its back-edge block sets).
        let loops = Loops::find_all(func, LoopOrder::InsideOut);
        let mut natural_loops: BTreeMap<BasicBlockId, BTreeSet<BasicBlockId>> = BTreeMap::new();
        for loop_ in &loops.yet_to_unroll {
            natural_loops.entry(loop_.header).or_default().extend(loop_.blocks.iter().copied());
        }

        let mut violations = Vec::new();
        for (header, blocks) in &natural_loops {
            // The latest position among the loop's blocks, and which block holds it. Loop blocks
            // unreachable from the entry have no position and are ignored.
            let Some((max_position, offending_loop_block)) = blocks
                .iter()
                .filter_map(|block| position.get(block).map(|pos| (*pos, *block)))
                .max_by_key(|(pos, _)| *pos)
            else {
                continue;
            };

            // The blocks reached by *forward*-exiting the loop: successors of a loop block that
            // are not themselves in the loop and are not an enclosing header reached by a
            // back-edge (such a header dominates the block and legitimately precedes the loop).
            // `BTreeSet` keeps the reported order deterministic.
            let mut exits: BTreeSet<BasicBlockId> = BTreeSet::new();
            for &block in blocks {
                for successor in cfg.successors(block) {
                    if !blocks.contains(&successor) && !loops.dom.dominates(successor, block) {
                        exits.insert(successor);
                    }
                }
            }

            for exit_block in exits {
                let Some(&exit_position) = position.get(&exit_block) else {
                    continue;
                };
                if exit_position <= max_position {
                    violations.push(Violation {
                        loop_header: *header,
                        exit_block,
                        offending_loop_block,
                        exit_position,
                        offending_position: max_position,
                    });
                }
            }
        }
        violations
    }

    /// Compile a Noir source string to fully-optimized SSA, mirroring the primary pass pipeline.
    ///
    /// The corpus programs are self-contained (no `std` imports, no operators on aggregate types)
    /// so the frontend test helper, which does not link the standard library, can compile them.
    /// Broad coverage over real `test_programs/` (which use the full stdlib) is exercised
    /// separately by running the property check inside the `nargo_cli` execution suite.
    fn optimized_ssa(src: &str) -> Ssa {
        let program = get_monomorphized(src).expect("program should monomorphize");
        let ssa = generate_ssa(program).expect("SSA generation should succeed");
        let options = SsaEvaluatorOptions::default();
        let builder = SsaBuilder::from_ssa(
            ssa,
            options.ssa_logging.clone(),
            options.ssa_logging_hide_unchanged,
            false,
            None,
        );
        builder.run_passes(&primary_passes(&options)).expect("passes should run").finish()
    }

    /// Self-contained programs with complex control flow whose loops survive into optimized SSA:
    /// `unconstrained` functions with runtime bounds (so the unroller cannot flatten them) and a
    /// mix of nested loops, `break`, `continue`, and early `return` — which is what produces the
    /// extra loop-exit blocks this ordering property targets.
    const CORPUS: &[(&str, &str)] = &[
        (
            "triple_nested_with_break_and_continue",
            "unconstrained fn main(n: u32, m: u32, k: u32) -> pub u32 {
                let mut acc: u32 = 0;
                for i in 0..n {
                    for j in 0..m {
                        if j == i {
                            continue;
                        }
                        for l in 0..k {
                            if l == 3 {
                                break;
                            }
                            acc += i * j + l;
                        }
                    }
                }
                acc
            }",
        ),
        (
            "sequential_loops_with_break",
            "unconstrained fn main(n: u32) -> pub u32 {
                let mut a: u32 = 0;
                for i in 0..n {
                    if i == 7 {
                        break;
                    }
                    a += i;
                }
                let mut b: u32 = a;
                for j in 0..n {
                    b += j * a;
                }
                b
            }",
        ),
        (
            "nested_loops_across_calls",
            "unconstrained fn inner(x: u32, bound: u32) -> u32 {
                let mut total: u32 = 0;
                for i in 0..bound {
                    if i == x {
                        break;
                    }
                    total += i;
                }
                total
            }
            unconstrained fn main(n: u32, m: u32) -> pub u32 {
                let mut acc: u32 = 0;
                for i in 0..n {
                    acc += inner(i, m);
                }
                acc
            }",
        ),
    ];

    #[test]
    fn loop_exits_follow_loop_bodies_in_optimized_ssa() {
        let mut functions_checked = 0;
        for (name, src) in CORPUS {
            let ssa = optimized_ssa(src);
            for func in ssa.functions.values() {
                let violations = find_loop_exit_ordering_violations(func);
                let forward = PostOrder::with_function(func).into_vec_reverse();
                assert!(
                    violations.is_empty(),
                    "program `{name}` fn `{}` ({:?}): loop blocks ordered after a loop exit.\nforward order: {forward:?}\nviolations: {violations:#?}\n{func}",
                    func.name(),
                    func.runtime(),
                );
                functions_checked += 1;
            }
        }

        // Guard against the corpus silently compiling to nothing (which would make the assertions
        // vacuous), and against losing all loops to optimization.
        assert!(
            functions_checked >= CORPUS.len(),
            "expected to check at least one function per corpus program, only checked {functions_checked}"
        );
    }
}
