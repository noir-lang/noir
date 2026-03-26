use std::collections::HashMap;

use super::value::ValueId;

/// Union-Find (disjoint set) data structure for efficiently computing connected components
/// over [`ValueId`]s. Uses path compression and union by rank for near-O(1) amortized operations.
pub(crate) struct UnionFind {
    parent: HashMap<ValueId, ValueId>,
    rank: HashMap<ValueId, u32>,
}

impl UnionFind {
    pub(crate) fn new() -> Self {
        Self { parent: HashMap::default(), rank: HashMap::default() }
    }

    /// Ensure a value exists in the union-find.
    pub(crate) fn make_set(&mut self, v: ValueId) {
        self.parent.entry(v).or_insert(v);
    }

    /// Check whether a value has been added to this union-find.
    pub(crate) fn contains(&self, v: &ValueId) -> bool {
        self.parent.contains_key(v)
    }

    /// Find the root representative of the set containing `v`, with path compression.
    pub(crate) fn find(&mut self, v: ValueId) -> ValueId {
        let p = self.parent[&v];
        if p == v {
            return v;
        }
        let root = self.find(p);
        self.parent.insert(v, root);
        root
    }

    /// Union the sets containing `a` and `b`. Uses union by rank.
    pub(crate) fn union(&mut self, a: ValueId, b: ValueId) {
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

    /// Union all values in the iterator into one set.
    pub(crate) fn union_all(&mut self, values: impl IntoIterator<Item = ValueId>) {
        let mut first = None;
        for v in values {
            self.make_set(v);
            match first {
                None => first = Some(v),
                Some(f) => self.union(f, v),
            }
        }
    }
}
