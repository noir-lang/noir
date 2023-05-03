//! The post-order for a given function represented as a vector of basic block ids.
//!
//! This ordering is beneficial to the efficiency of various algorithms, such as those for dead
//! code elimination and calculating dominance trees.

use std::collections::HashSet;

use crate::ssa_refactor::ir::{basic_block::BasicBlockId, function::Function};

/// Depth-first traversal stack state marker for computing the cfg post-order.
enum Visit {
    First,
    Last,
}

pub(crate) struct PostOrder(Vec<BasicBlockId>);

impl PostOrder {
    pub(crate) fn as_slice(&self) -> &[BasicBlockId] {
        self.0.as_slice()
    }
}

impl PostOrder {
    /// Allocate and compute a function's block post-order. Pos
    pub(crate) fn with_function(func: &Function) -> Self {
        PostOrder(Self::compute_post_order(func))
    }

    // Computes the post-order of the function by doing a depth-first traversal of the
    // function's entry block's previously unvisited children. Each block is sequenced according
    // to when the traversal exits it.
    fn compute_post_order(func: &Function) -> Vec<BasicBlockId> {
        let mut stack = vec![(Visit::First, func.entry_block())];
        let mut visited: HashSet<BasicBlockId> = HashSet::new();
        let mut post_order: Vec<BasicBlockId> = Vec::new();

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
                        for successor_id in func.dfg[block_id].successors().rev() {
                            if !visited.contains(&successor_id) {
                                // This not visited check would also be cover by the the next
                                // iteration, but checking here two saves an iteration per successor.
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
    use crate::ssa_refactor::ir::{
        function::Function, instruction::TerminatorInstruction, map::Id, post_order::PostOrder,
        types::Type,
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
        let mut func = Function::new("func".into(), func_id);
        let block_a_id = func.entry_block();
        let block_b_id = func.dfg.make_block();
        let block_c_id = func.dfg.make_block();
        let block_d_id = func.dfg.make_block();
        let block_e_id = func.dfg.make_block();
        let block_f_id = func.dfg.make_block();

        // A → B   •
        // ↓
        // D   •   •
        let cond_a = func.dfg.add_block_parameter(block_a_id, Type::unsigned(1));
        func.dfg.set_block_terminator(
            block_a_id,
            TerminatorInstruction::JmpIf {
                condition: cond_a,
                then_destination: block_b_id,
                else_destination: block_d_id,
            },
        );
        //  •   B   •
        //  •   ↓   •
        //  •   E   •
        func.dfg.set_block_terminator(
            block_b_id,
            TerminatorInstruction::Jmp { destination: block_e_id, arguments: vec![] },
        );
        // •   •   •
        //
        // D ← E → F
        let cond_e = func.dfg.add_block_parameter(block_e_id, Type::unsigned(1));
        func.dfg.set_block_terminator(
            block_e_id,
            TerminatorInstruction::JmpIf {
                condition: cond_e,
                then_destination: block_d_id,
                else_destination: block_f_id,
            },
        );
        // •   B   •
        //   ↗
        // D   •   •
        func.dfg.set_block_terminator(
            block_d_id,
            TerminatorInstruction::Jmp { destination: block_b_id, arguments: vec![] },
        );
        // •   •   C
        // •   •   ↓
        // •   •   F
        func.dfg.set_block_terminator(
            block_c_id,
            TerminatorInstruction::Jmp { destination: block_f_id, arguments: vec![] },
        );

        let post_order = PostOrder::with_function(&func);
        assert_eq!(post_order.0, [block_d_id, block_f_id, block_e_id, block_b_id, block_a_id]);
    }
}
