//! Union-Find (disjoint set) data structure for efficiently computing connected components.
//! Uses path compression and union by rank for near-O(1) amortized operations.

use rustc_hash::FxHashMap as HashMap;

use std::hash::Hash;

pub(crate) struct UnionFind<K: Copy + Eq + Hash> {
    parent: HashMap<K, K>,
    rank: HashMap<K, u32>,
}

impl<K: Copy + Eq + Hash> Default for UnionFind<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Copy + Eq + Hash> UnionFind<K> {
    pub(crate) fn new() -> Self {
        Self { parent: HashMap::default(), rank: HashMap::default() }
    }

    /// Ensure a value exists in the union-find.
    pub(crate) fn make_set(&mut self, v: K) {
        self.parent.entry(v).or_insert(v);
    }

    /// Find the root representative of the set containing `v`, with path compression.
    pub(crate) fn find(&mut self, v: K) -> K {
        // Lazily initialize the set for `v` as himself if it doesn't exist.
        let p = *self.parent.entry(v).or_insert(v);
        if p == v {
            return v;
        }
        let root = self.find(p);
        self.parent.insert(v, root);
        root
    }

    /// Find the root representative of the set containing `v`, without path
    /// compression.
    fn find_immutable(&self, v: K) -> Option<K> {
        let mut current = v;
        loop {
            let parent = *self.parent.get(&current)?;
            if parent == current {
                return Some(current);
            }
            current = parent;
        }
    }

    /// Return a map from each class representative to the number of values
    /// in that class.
    pub(crate) fn class_sizes(&self) -> HashMap<K, u32> {
        let mut sizes = HashMap::default();
        for &v in self.parent.keys() {
            if let Some(root) = self.find_immutable(v) {
                *sizes.entry(root).or_insert(0) += 1;
            }
        }
        sizes
    }

    /// Union the sets containing `a` and `b`. Uses union by rank.
    pub(crate) fn union(&mut self, a: K, b: K) {
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
/// - A vec of all members in each group.
pub(crate) fn connected_components<K: Copy + Eq + Hash>(
    edges: &HashMap<K, K>,
) -> (HashMap<K, usize>, Vec<Vec<K>>) {
    let mut uf = UnionFind::new();
    for (&k, &v) in edges {
        uf.make_set(k);
        uf.make_set(v);
        uf.union(k, v);
    }

    let all_values: Vec<K> = uf.parent.keys().copied().collect();
    let mut root_to_group: HashMap<K, usize> = HashMap::default();
    let mut groups: HashMap<K, usize> = HashMap::default();
    let mut group_members: Vec<Vec<K>> = Vec::new();

    for value in all_values {
        let root = uf.find(value);
        let group_id = *root_to_group.entry(root).or_insert_with(|| {
            group_members.push(Vec::new());
            group_members.len() - 1
        });
        groups.insert(value, group_id);
        group_members[group_id].push(value);
    }

    (groups, group_members)
}
