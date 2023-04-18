use super::basic_blocks::BasicBlock;

// Given an entry node N0
// A node A is said to dominate node B, if every
// path from N0 to A must go through B

/// ControlFlowGraph specifies the control flow of a program
/// The nodes in the graph will be basic blocks and an
/// edge from BasicBlock1 to BasicBlock2 signifies that
/// there is a way for control to be given to BasicBlock2
/// from BasicBlock1
use petgraph::{
    graph::{DiGraph, NodeIndex, WalkNeighbors},
    stable_graph::IndexType,
    Direction,
};

#[derive(Debug, Default)]
pub struct ControlFlowGraph {
    // Graph where the Nodes have data `BasicBlock`
    // and edges have data `()`
    graph: DiGraph<BasicBlock, ()>,
    // The first block in the list of basic blocks
    // also referred to as the entry block.
    //
    // This block has no predecessors.
    entry_block: Option<BasicBlockId>,
}

impl ControlFlowGraph {
    pub fn with_block(entry_block: BasicBlock) -> (BasicBlockId, Self) {
        let mut cfg = ControlFlowGraph { graph: DiGraph::default(), entry_block: None };
        let entry_block_id = cfg.add_entry_block(entry_block);
        (entry_block_id, cfg)
    }

    pub fn add_entry_block(&mut self, entry_block: BasicBlock) -> BasicBlockId {
        assert!(self.entry_block.is_none(), "cannot set the root block twice");

        let block_id = self.add_basic_block(entry_block);
        self.entry_block = Some(block_id);

        block_id
    }

    pub fn add_edge(&mut self, from: BasicBlockId, to: BasicBlockId) {
        self.graph.add_edge(from.0, to.0, ());
    }

    pub fn add_basic_block(&mut self, block_id: BasicBlock) -> BasicBlockId {
        BasicBlockId(self.graph.add_node(block_id))
    }
    pub fn get_mut(&mut self, block_id: BasicBlockId) -> Option<&mut BasicBlock> {
        self.graph.node_weight_mut(block_id.0)
    }
    pub fn get(&mut self, block_id: BasicBlockId) -> Option<&BasicBlock> {
        self.graph.node_weight(block_id.0)
    }

    // TODO: do we ever need to merge basic blocks?
    // pub fn merge_blocks()

    pub fn successors(&self, block_id: BasicBlockId) -> Neighbors {
        Neighbors { inner: self.graph.neighbors_directed(block_id.0, Direction::Outgoing).detach() }
    }

    pub fn predecessors(&self, block_id: BasicBlockId) -> Neighbors {
        Neighbors { inner: self.graph.neighbors_directed(block_id.0, Direction::Incoming).detach() }
    }

    pub fn frontier(&mut self, block_id: BasicBlockId) {
        use petgraph::algo::dominators::simple_fast;
        let entry_block = self.entry_block.expect("entry block is not set");
        simple_fast(&self.graph, entry_block.0);
    }
}

pub struct Neighbors {
    inner: WalkNeighbors<u32>,
}

impl Neighbors {
    pub fn next_node(&mut self, cfg: &ControlFlowGraph) -> Option<BasicBlockId> {
        self.inner.next_node(&cfg.graph).map(|node_index| BasicBlockId(node_index))
    }
}
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub struct BasicBlockId(NodeIndex);

fn foo() {
    use petgraph::algo::{dijkstra, min_spanning_tree};
    use petgraph::data::FromElements;
    use petgraph::dot::{Config, Dot};
    #[derive(Debug, Default)]
    struct Foo;
    #[derive(Debug, Default)]
    struct Bar;
    // Create an undirected graph with `i32` nodes and edges with `()` associated data.
    let mut g = DiGraph::<Foo, Bar>::new();
    let n = g.add_node(Foo);

    g.add_edge(n, n, Bar);
}

#[test]
fn tfoo() {
    foo()
}
