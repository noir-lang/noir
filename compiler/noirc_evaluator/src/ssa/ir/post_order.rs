//! The post-order for a given function represented as a vector of basic block ids.
//!
//! This ordering is beneficial to the efficiency of various algorithms, such as those for dead
//! code elimination and calculating dominance trees.

use std::collections::HashSet;

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
    /// broken by block id. A loop's exit is additionally ranked by the maximum cost *within the
    /// loop it exits*, so loop bodies stay contiguous and an exit follows the entire loop (this
    /// cascades through nested loops).
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
    /// visited only after all of its non-back-edge predecessors, and a loop's exit is visited
    /// only after every block of the loop it exits.
    ///
    /// A plain depth-first post-order already visits successors before predecessors for
    /// back-edge-free CFGs, but it can order a loop exit ahead of the loop body (see
    /// <https://github.com/noir-lang/noir/issues/9771>). To get a deterministic, well-behaved
    /// order we rank each block by the *maximum cost to reach it* from a root: the length of the
    /// longest path once back-edges are removed. Sorting by `(cost, block id)` is always a valid
    /// topological order of the back-edge-free CFG, since any forward edge `u -> v` forces
    /// `cost(v) >= cost(u) + 1`.
    ///
    /// The forward-path cost alone is not enough for loops whose exit branches directly off the
    /// header: such an exit sits one step from the header yet should follow the whole loop body,
    /// which can extend much deeper. To account for this we add a virtual forward edge from every
    /// block of a loop to each block reached on leaving the loop, so the exit's cost is forced
    /// past the maximum cost within the loop. The longest path over this augmented DAG keeps loop
    /// bodies contiguous and orders exits after the entire loop, cascading correctly through
    /// nested loops.
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

        // Augmented predecessor sets, seeded with the forward CFG edges. Back-edge predecessors
        // (larger `rpo_index`) and predecessors unreachable from the roots (no `rpo_index`) are
        // excluded. Loop-exit virtual edges are added below.
        let mut predecessors: HashMap<BasicBlockId, HashSet<BasicBlockId>> = HashMap::default();
        for &block in &post_order {
            let block_index = rpo_index[&block];
            let block_predecessors = predecessors.entry(block).or_default();
            for predecessor in cfg.predecessors(block) {
                if rpo_index.get(&predecessor).is_some_and(|&index| index < block_index) {
                    block_predecessors.insert(predecessor);
                }
            }
        }

        // Group each loop's back edges by header. A back edge `start -> header` is a predecessor
        // with a larger `rpo_index` than the block it points at.
        let mut back_edges: HashMap<BasicBlockId, Vec<BasicBlockId>> = HashMap::default();
        for &block in &post_order {
            let block_index = rpo_index[&block];
            for predecessor in cfg.predecessors(block) {
                if rpo_index.get(&predecessor).is_some_and(|&index| index > block_index) {
                    back_edges.entry(block).or_default().push(predecessor);
                }
            }
        }

        // For each loop, add a virtual forward edge from every block in the loop to each block
        // reached on leaving it, forcing the exit's cost past the maximum cost within the loop.
        for (&header, back_edge_starts) in &back_edges {
            let body = Self::loop_body(cfg, header, back_edge_starts);
            let mut exits: HashSet<BasicBlockId> = HashSet::default();
            for &block in &body {
                for successor in cfg.successors(block) {
                    if !body.contains(&successor) {
                        exits.insert(successor);
                    }
                }
            }
            for &exit in &exits {
                let exit_predecessors =
                    predecessors.get_mut(&exit).expect("exit is reachable from a root");
                exit_predecessors.extend(body.iter().copied());
            }
        }

        let cost = Self::longest_paths(&post_order, &rpo_index, cfg, &predecessors);

        // Order by ascending cost (ties broken by block id) to get the reverse-post-order, then
        // reverse it to recover the post-order.
        let mut reverse_post_order = post_order;
        reverse_post_order.reverse();
        reverse_post_order.sort_by_key(|block| (cost[block], *block));
        reverse_post_order.reverse();
        reverse_post_order
    }

    /// The blocks of the natural loop whose header is `header` and whose back edges originate at
    /// `back_edge_starts`: the header together with every block that can reach a back-edge start
    /// without passing back through the header.
    ///
    /// Mirrors `Loop::find_blocks_in_loop` but is kept local to `post_order` so the IR layer does
    /// not depend on the optimization passes (and to avoid recursing through `PostOrder`, which
    /// `Loops::find_all` itself uses).
    fn loop_body(
        cfg: &ControlFlowGraph,
        header: BasicBlockId,
        back_edge_starts: &[BasicBlockId],
    ) -> HashSet<BasicBlockId> {
        // Insert the header first so the backward walk does not go past it.
        let mut body: HashSet<BasicBlockId> = HashSet::default();
        body.insert(header);

        let mut stack = Vec::new();
        for &start in back_edge_starts {
            if body.insert(start) {
                stack.push(start);
            }
        }
        while let Some(block) = stack.pop() {
            for predecessor in cfg.predecessors(block) {
                if body.insert(predecessor) {
                    stack.push(predecessor);
                }
            }
        }
        body
    }

    /// The maximum cost to reach each block: `cost(v) = max(cost(u) + 1)` over the augmented
    /// predecessors `u` of `v`, or 0 for a root. Computed as a longest-path DP over the augmented
    /// DAG, visiting blocks in a topological order (Kahn's algorithm) so every predecessor's cost
    /// is known first.
    fn longest_paths(
        post_order: &[BasicBlockId],
        rpo_index: &HashMap<BasicBlockId, u32>,
        cfg: &ControlFlowGraph,
        predecessors: &HashMap<BasicBlockId, HashSet<BasicBlockId>>,
    ) -> HashMap<BasicBlockId, u32> {
        let mut successors: HashMap<BasicBlockId, Vec<BasicBlockId>> = HashMap::default();
        let mut in_degree: HashMap<BasicBlockId, usize> = HashMap::default();
        for &block in post_order {
            successors.entry(block).or_default();
        }
        for (&block, block_predecessors) in predecessors {
            in_degree.insert(block, block_predecessors.len());
            for &predecessor in block_predecessors {
                successors.entry(predecessor).or_default().push(block);
            }
        }

        let mut cost: HashMap<BasicBlockId, u32> = HashMap::default();
        let mut ready: Vec<BasicBlockId> =
            in_degree.iter().filter(|&(_, &degree)| degree == 0).map(|(&block, _)| block).collect();
        while let Some(block) = ready.pop() {
            let block_cost = predecessors[&block]
                .iter()
                .map(|predecessor| cost[predecessor] + 1)
                .max()
                .unwrap_or(0);
            cost.insert(block, block_cost);
            for &successor in &successors[&block] {
                let degree = in_degree.get_mut(&successor).expect("successor has an in-degree");
                *degree -= 1;
                if *degree == 0 {
                    ready.push(successor);
                }
            }
        }

        // A block left uncosted can only arise from an irreducible cycle in the augmented graph.
        // Fall back to the plain forward-path cost (in reverse-post-order) so every block still
        // gets a value and the result remains a valid topological order of the forward CFG.
        if cost.len() < post_order.len() {
            for &block in post_order.iter().rev() {
                if cost.contains_key(&block) {
                    continue;
                }
                let block_index = rpo_index[&block];
                let block_cost = cfg
                    .predecessors(block)
                    .filter_map(|predecessor| {
                        let predecessor_index = *rpo_index.get(&predecessor)?;
                        (predecessor_index < block_index)
                            .then(|| cost.get(&predecessor).copied().unwrap_or(0) + 1)
                    })
                    .max()
                    .unwrap_or(0);
                cost.insert(block, block_cost);
            }
        }

        cost
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
        // Blocks are ordered by their maximum cost to reach them, where a loop's exit is ranked
        // by the maximum cost within the loop it exits (a virtual edge from every loop block to
        // its exit target). The inner loop {b4, b5, b7, b8} forces its exit b6 past b8, and the
        // outer loop {b1, b2, b4, b5, b6, b7, b8} forces its exit b3 past all of them:
        //
        //   b0=0, b1=1, b2=2, b4=3, b5=4, b7=5, b8=6, b6=7, b3=8
        //
        // Sorting by (cost, block-id) ascending gives the reverse-post-order
        // [b0, b1, b2, b4, b5, b7, b8, b6, b3]. Every block is still visited only after all of its
        // (non-back-edge) predecessors, which is what #9771 requires, and the loop bodies are now
        // contiguous with their exits ordered last. The expected post-order is the reverse:
        let expected_post_order = [b(3), b(6), b(8), b(7), b(5), b(4), b(2), b(1), b(0)];

        let ssa = Ssa::from_str(src).unwrap();

        let func = ssa.main();
        let post_order = PostOrder::with_function(func);
        assert_eq!(post_order.0, expected_post_order);
    }
}
