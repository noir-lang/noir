//! The post-order for a given function represented as a vector of basic block ids.
//!
//! Post-order is defined here as the reverse-topological sort of the strongly-connected components
//! of the control-flow graph. Any blocks within a strongly-connected component (a loop) will be in
//! an arbitrary order.
//!
//! This ordering is beneficial to the efficiency of various algorithms, such as those for dead
//! code elimination and calculating dominance trees.
use iter_extended::vecmap;
use petgraph::{Direction, Graph, graph::NodeIndex, visit::EdgeRef};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

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

    /// Return blocks in reverse-post-order a.k.a. forward-order.
    pub(crate) fn into_vec_reverse(self) -> Vec<BasicBlockId> {
        let mut blocks = self.into_vec();
        blocks.reverse();
        blocks
    }

    /// Finds the maximum cost from `root` to every reachable node in a directed graph.
    /// Each edge may be used at most once.
    fn topological_sort(graph: &Graph<(), ()>) -> Vec<NodeIndex> {
        petgraph::algo::kosaraju_scc(graph)
            .into_iter()
            .rev()
            .flat_map(|scc| {
                if scc.len() == 1 {
                    scc
                // Build subgraph of just the loop blocks
                } else if let Some((subgraph, new_to_old_indices)) =
                    Self::loop_subgraph(graph, &scc)
                {
                    let mut hops = Self::topological_sort(&subgraph);
                    hops.iter_mut().for_each(|index| *index = new_to_old_indices[index]);
                    hops
                } else {
                    scc
                }
            })
            .collect()
    }

    fn loop_subgraph(
        graph: &Graph<(), ()>,
        scc: &[NodeIndex],
    ) -> Option<(Graph<(), ()>, HashMap<NodeIndex, NodeIndex>)> {
        let mut new_graph = Graph::new();
        let mut old_to_new = HashMap::default();
        let mut new_to_old = HashMap::default();

        for node in scc {
            let new_node = new_graph.add_node(());
            old_to_new.insert(*node, new_node);
            new_to_old.insert(new_node, *node);
        }

        // blocks that are candidates for the loop header.
        // This is normally a straightforward answer of "the only block with an outgoing
        // edge leading out of the loop" but in the presense of loops with `break`, there
        // may be multiple such blocks. In those cases, the loop header is the candidate
        // which still leads to all other blocks in the loops.
        let mut possible_headers = HashSet::default();
        for node in scc {
            let new_node = old_to_new[node];

            for neighbor in graph.neighbors_directed(*node, Direction::Outgoing) {
                let Some(new_neighbor) = old_to_new.get(&neighbor) else {
                    possible_headers.insert(new_node);
                    continue;
                };
                new_graph.add_edge(new_node, *new_neighbor, ());
            }
        }

        // This can occur with infinite loops since they may have no exits
        if possible_headers.is_empty() {
            return None;
        }

        // If there are multiple possible headers in the loop, arbitrarily choose the first
        let header = possible_headers.into_iter().next().unwrap();

        // Finally, cut all incoming edges on the header
        let edges = vecmap(new_graph.edges_directed(header, Direction::Incoming), |edge| edge.id());
        for edge in edges {
            new_graph.remove_edge(edge);
        }

        Some((new_graph, new_to_old))
    }

    // Computes the post-order of the CFG which is defined as the reverse-topological sort
    // of the SCCs of the CFG graph where each SCC is recursively sorted in reverse-topological
    // order. See the comment in `tests::nested_loop` for an example.
    fn compute_post_order(cfg: &ControlFlowGraph) -> Vec<BasicBlockId> {
        let mut order = Self::compute_topological_order(cfg);
        order.reverse();
        order
    }

    pub(super) fn compute_topological_order(cfg: &ControlFlowGraph) -> Vec<BasicBlockId> {
        // Implementation note:
        // - Computing via DfsPostOrder is invalid since in the presence of loops, the exit
        //   block may not be the first within the post-order.
        // - Computing via SCCS is invalid since `kosaraju_scc` doesn't specify an ordering
        //   within SCCS.
        //
        // Observation: the topological order for a SSA graph with 1 entry and 1 exit
        // is each block sorted by order of the maximum cost to reach that block from the
        // entry block using each edge at most once.
        let graph = cfg.as_petgraph();
        let sort = Self::topological_sort(&graph.graph);
        vecmap(sort, |index| graph.node_to_block[&index])
    }
}

#[cfg(test)]
mod tests {
    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            basic_block::BasicBlockId, function::Function, map::Id, post_order::PostOrder,
            types::Type,
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
        // F, B, D, E, A

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
        assert_eq!(post_order.0, [block_f_id, block_b_id, block_d_id, block_e_id, block_a_id]);
    }

    #[test]
    fn loop_regression() {
        // b0 -> b1 <-> b2
        //        |
        //        V
        //       b3
        let src = "
            acir(inline) fn main f0 {
              b0():
                jmp b1(u1 0)
              b1(v0: u1):
                jmpif v0 then: b2, else: b3
              b2():
                jmp b1(v0)
              b3():
                return
            }
            ";

        let ssa = Ssa::from_str(src).unwrap();

        let func = ssa.main();
        let post_order = PostOrder::with_function(func);
        assert_eq!(post_order.0, [b(3), b(2), b(1), b(0)]);
    }

    #[test]
    fn loop_with_different_indices() {
        // Ensure block numbering doesn't stop b2 from being the first in post-order
        // b0 -> b1 <-> b3
        //        |
        //        V
        //       b2
        let src = "
            brillig(inline) fn main f0 {
              b0(v0: u1):
                jmp b1(v0)
              b1(v1: u1):
                jmpif v1 then: b9, else: b2
              b2():
                return
              b9():
                jmp b1(v0)
            }
            ";

        let ssa = Ssa::from_str(src).unwrap();

        let func = ssa.main();
        let post_order = PostOrder::with_function(func);
        assert_eq!(post_order.0, [b(2), b(3), b(1), b(0)]);
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
