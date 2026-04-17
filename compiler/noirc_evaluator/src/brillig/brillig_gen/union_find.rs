//! Union-Find (disjoint set) data structure for efficiently computing connected components.
//! Uses path compression and union by rank for near-O(1) amortized operations.

use rustc_hash::FxHashMap as HashMap;

use crate::ssa::ir::value::ValueId;

pub(super) struct UnionFind {
    parent: HashMap<ValueId, ValueId>,
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

/// Compute connected components from a directed edge map.
///
/// Returns:
/// - A map from each value to its group ID.
/// - A vec of live counts per group (initialized to group size).
pub(super) fn connected_components(
    edges: &HashMap<ValueId, ValueId>,
) -> (HashMap<ValueId, usize>, Vec<usize>) {
    let mut uf = UnionFind::new();
    for (&k, &v) in edges {
        uf.make_set(k);
        uf.make_set(v);
        uf.union(k, v);
    }

    let all_values: Vec<ValueId> = uf.parent.keys().copied().collect();
    let mut root_to_group: HashMap<ValueId, usize> = HashMap::default();
    let mut groups: HashMap<ValueId, usize> = HashMap::default();
    let mut group_live_counts: Vec<usize> = Vec::new();

    for value in all_values {
        let root = uf.find(value);
        let group_id = *root_to_group.entry(root).or_insert_with(|| {
            group_live_counts.push(0);
            group_live_counts.len() - 1
        });
        groups.insert(value, group_id);
        group_live_counts[group_id] += 1;
    }

    (groups, group_live_counts)
}
