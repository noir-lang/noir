//! The post-order for a given function represented as a vector of basic block ids.
//!
//! Post-order is defined here as the reverse-topological sort of the strongly-connected components
//! of the control-flow graph. Any blocks within a strongly-connected component (a loop) will be in
//! an arbitrary order.
//!
//! This ordering is beneficial to the efficiency of various algorithms, such as those for dead
//! code elimination and calculating dominance trees.
use petgraph::visit::EdgeRef;

use crate::ssa::ir::{basic_block::BasicBlockId, function::Function};

use super::cfg::ControlFlowGraph;

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
        PostOrder::with_cfg(&cfg)
    }

    /// Allocate and compute a function's block post-order.
    pub(crate) fn with_cfg(cfg: &ControlFlowGraph) -> Self {
        PostOrder(Self::compute_post_order(&cfg))
    }

    /// Return blocks in post-order.
    pub(crate) fn into_vec(self) -> Vec<BasicBlockId> {
        self.0
    }

    /// Return blocks in reverse-post-order a.k.a. forward-order.
    pub(crate) fn into_vec_reverse(self) -> Vec<BasicBlockId> {
        let mut blocks = self.into_vec();
        blocks.reverse();
        blocks
    }

    // Computes the post-order of the CFG which is defined as the reverse-topological sort
    // of the SCCs of the CFG graph where each SCC is recursively sorted in reverse-topological
    // order. See the comment in `tests::nested_loop` for an example.
    fn compute_post_order2(cfg: &ControlFlowGraph) -> Vec<BasicBlockId> {
        // Assumption: jumping to a lower block number = loop back-edge
        let mut graph = cfg.as_petgraph();

        // Remove back-edges of loops
        for block in graph.block_to_node.keys() {
            let mut predecessors = cfg.predecessors(*block);
            if predecessors.len() > 1 {
                // If neighbor is a back-edge from a loop
                if let Some(neighbor) = predecessors.find(|neighbor| neighbor > block) {
                    let block_node = graph.block_to_node[block];
                    let neighbor_node = graph.block_to_node[&neighbor];

                    // Remove the back-edge
                    let mut edges = graph.graph.edges_connecting(neighbor_node, block_node);
                    let edge = edges.next().expect("Expected back-edge in cfg");
                    graph.graph.remove_edge(edge.id());

                    // And replace it with a link to after the loop.
                    // We guess the end of the loop is the successor with the largest block id,
                    // although this assumption may not be true. If it is not then we may get
                    // some odd loop block ordering (and they may potentiall be ordered after
                    // the rest of the program).
                    if let Some(loop_end) = cfg.successors(*block).max() {
                        let loop_end = graph.block_to_node[&loop_end];
                        graph.graph.update_edge(neighbor_node, loop_end, ());
                    }
                }
            }
        }

        // Graph should be closer to (but not necessarily) a DAG now
        let entry = graph.block_to_node[&cfg.find_entry_block()];
        let mut dfs = petgraph::visit::DfsPostOrder::new(&graph.graph, entry);
        let mut order = Vec::with_capacity(graph.block_to_node.len());

        while let Some(block) = dfs.next(&graph.graph) {
            order.push(graph.node_to_block[&block]);
        }

        order
    }

    // Computes the post-order of the CFG which is defined as the reverse-topological sort
    // of the SCCs of the CFG graph where each SCC is recursively sorted in reverse-topological
    // order. See the comment in `tests::nested_loop` for an example.
    fn compute_post_order(cfg: &ControlFlowGraph) -> Vec<BasicBlockId> {
        // Assumption: jumping to a lower block number = loop back-edge
        let graph = cfg.as_petgraph();
        let sccs = petgraph::algo::kosaraju_scc(&graph.graph);
        sccs.into_iter()
            .flat_map(|scc| scc.into_iter().rev().map(|node| graph.node_to_block[&node]))
            .collect()
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

    /// Helper to construct a BasicBlockId with a syntax resembling the `b0`
    /// syntax used in comments/ssa output.
    fn b(id: u32) -> BasicBlockId {
        BasicBlockId::test_new(id)
    }

    #[test]
    fn single_block() {
        let func_id = Id::test_new(0);
        let func = Function::new("func".into(), func_id);
        let post_order = PostOrder::with_function(&func);
        assert_eq!(post_order.0, [func.entry_block()]);
    }

    #[test]
    fn arb_graph_with_unreachable() {
        // This test is a bit odd since we'll never produce programs like this
        // A → B   C
        // ↓ ↗ ↓   ↓
        // D ← E → F
        // (`A` is entry block)
        // Result:
        // F, B, E, D, A

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
        assert_eq!(post_order.0, [block_f_id, block_b_id, block_e_id, block_d_id, block_a_id]);
    }

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
        assert_eq!(post_order.0, [b3, b2, b1, b0]);
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
        // And the expected post-order is simply the reverse of this topological ordering
        let expected_post_order = [b(3), b(6), b(8), b(7), b(5), b(4), b(2), b(1), b(0)];

        let ssa = Ssa::from_str(src).unwrap();

        let func = ssa.main();
        let post_order = PostOrder::with_function(func);
        assert_eq!(post_order.0, expected_post_order);
    }
}
