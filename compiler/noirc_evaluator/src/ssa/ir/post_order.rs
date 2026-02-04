//! The post-order for a given function represented as a vector of basic block ids.
//!
//! This ordering is beneficial to the efficiency of various algorithms, such as those for dead
//! code elimination and calculating dominance trees.

use std::collections::HashSet;

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
        PostOrder(Self::compute_post_order(cfg))
    }

    /// Return blocks in post-order.
    pub(crate) fn into_vec(self) -> Vec<BasicBlockId> {
        self.0
    }

    /// Return blocks in reverse-post-order (RPO).
    ///
    /// In RPO, each block is visited before any of its successors.
    /// Notably, this is not the same as topological sorting.
    ///
    /// Take this CFG for example:
    /// ```text
    ///      b0
    ///      |
    ///      b1<-+
    ///     /  \ |
    ///    b3   b2
    /// ```
    /// Intuitively we would like to see `[b0, b1, b2, b3]`,
    /// but the actual RPO is `[b0, b1, b3, b2]`.
    pub(crate) fn into_vec_reverse(self) -> Vec<BasicBlockId> {
        let mut blocks = self.into_vec();
        blocks.reverse();
        blocks
    }

    // Computes the post-order of the CFG by doing a depth-first traversal of the
    // function's entry block's previously unvisited children. Each block is sequenced according
    // to when the traversal exits it.
    fn compute_post_order(cfg: &ControlFlowGraph) -> Vec<BasicBlockId> {
        let mut stack = vec![];
        let mut visited: HashSet<BasicBlockId> = HashSet::new();
        let mut post_order: Vec<BasicBlockId> = Vec::new();

        // Set root blocks
        stack.extend(cfg.compute_entry_blocks().into_iter().map(|root| (Visit::First, root)));

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
        builder.terminate_with_jmpif(cond_a, block_b_id, block_d_id);
        //  •   B   •
        //  •   ↓   •
        //  •   E   •
        builder.switch_to_block(block_b_id);
        builder.terminate_with_jmp(block_e_id, vec![]);
        // •   •   •
        //
        // D ← E → F
        builder.switch_to_block(block_e_id);
        builder.terminate_with_jmpif(cond_e, block_d_id, block_f_id);
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
        assert_eq!(post_order.0, [block_d_id, block_f_id, block_e_id, block_b_id, block_a_id]);
    }

    /// Helper to construct a BasicBlockId with a syntax resembling the `b0`
    /// syntax used in comments/ssa output.
    fn b(id: u32) -> BasicBlockId {
        BasicBlockId::test_new(id)
    }

    /// Documents the somewhat odd behavior from https://github.com/noir-lang/noir/issues/9771
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
        builder.terminate_with_jmpif(zero, b2, b3);

        builder.switch_to_block(b2);
        builder.terminate_with_jmp(b1, Vec::new());

        builder.switch_to_block(b3);
        builder.terminate_with_return(Vec::new());
        let ssa = builder.finish();

        let func = ssa.main();
        let post_order = PostOrder::with_function(func);

        // [3, 2, 1, 0] would be the ideal but we currently get the following:
        assert_eq!(post_order.0, [b2, b3, b1, b0]);
    }

    #[test]
    fn simple_if() {
        let src = "
        acir(inline) fn factorial f1 {
          b0(v1: u32):
            v2 = lt v1, u32 1
            jmpif v2 then: b1, else: b2
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
        // Ordering of b1, b2 is arbitrary
        assert_eq!(post_order.0, [b(3), b(1), b(2), b(0)]);
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
                jmpif v1 then: b2, else: b3
              b2():
                jmp b4(v1)
              b3():
                return
              b4(v2: u1):
                jmpif v2 then: b5, else: b6
              b5():
                jmpif v2 then: b7, else: b8
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
        // And finally to the original program to get:
        //
        // [b0, b1, b2, b4, b5, b7, b8, b6, b3]
        //
        // And the expected post-order is simply the reverse of this topological ordering:
        //
        // [b3, b6, b8, b7, b5, b4, b2, b1, b0]
        //
        // But we currently get:
        let expected_post_order = [b(8), b(7), b(5), b(6), b(4), b(2), b(3), b(1), b(0)];

        let ssa = Ssa::from_str(src).unwrap();

        let func = ssa.main();
        let post_order = PostOrder::with_function(func);
        assert_eq!(post_order.0, expected_post_order);
    }
}
