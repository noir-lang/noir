//! Union-Find (disjoint set) data structure for efficiently computing connected components.
//! Uses path compression and union by rank for near-O(1) amortized operations.

use rustc_hash::FxHashMap as HashMap;

use crate::ssa::ir::value::ValueId;

pub(super) struct UnionFind {
    pub(super) parent: HashMap<ValueId, ValueId>,
    rank: HashMap<ValueId, u32>,
}

impl UnionFind {
    pub(super) fn new() -> Self {
        Self { parent: HashMap::default(), rank: HashMap::default() }
    }

    /// Ensure a value exists in the union-find.
    pub(super) fn make_set(&mut self, v: ValueId) {
        self.parent.entry(v).or_insert(v);
    }

    /// Find the root representative of the set containing `v`, with path compression.
    pub(super) fn find(&mut self, v: ValueId) -> ValueId {
        let p = self.parent[&v];
        if p == v {
            return v;
        }
        let root = self.find(p);
        self.parent.insert(v, root);
        root
    }

    /// Union the sets containing `a` and `b`. Uses union by rank.
    pub(super) fn union(&mut self, a: ValueId, b: ValueId) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return;
        }
        let rank_a = *self.rank.entry(ra).or_insert(0);
        let rank_b = *self.rank.entry(rb).or_insert(0);
        if rank_a < rank_b {
            self.parent.insert(ra, rb);
        } else {
            self.parent.insert(rb, ra);
            if rank_a == rank_b {
                *self.rank.entry(ra).or_insert(0) += 1;
            }
        }
    }
}
