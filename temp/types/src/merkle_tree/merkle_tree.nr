use crate::{hash::merkle_hash, traits::Empty};

pub struct MerkleTree<let N: u32> {
    pub leaves: [Field; N],
    pub nodes: [Field; N],
}

impl<let N: u32> Empty for MerkleTree<N> {
    fn empty() -> Self {
        MerkleTree { leaves: [0; N], nodes: [0; N] }
    }
}

impl<let N: u32> MerkleTree<N> {
    pub fn new(leaves: [Field; N]) -> Self {
        let mut nodes = [0; N];

        // We need one less node than leaves, but we cannot have computed array lengths
        let total_nodes = N - 1;
        let half_size = N / 2;

        // hash base layer
        for i in 0..half_size {
            nodes[i] = merkle_hash(leaves[2 * i], leaves[2 * i + 1]);
        }

        // hash the other layers
        for i in 0..(total_nodes - half_size) {
            nodes[half_size + i] = merkle_hash(nodes[2 * i], nodes[2 * i + 1]);
        }

        MerkleTree { leaves, nodes }
    }

    pub fn get_root(self) -> Field {
        self.nodes[N - 2]
    }

    pub fn sibling_index(index: u32) -> u32 {
        if index % 2 == 0 {
            index + 1
        } else {
            index - 1
        }
    }

    pub fn get_sibling_path<let K: u32>(self, leaf_index: u32) -> [Field; K] {
        assert_eq(2.pow_32(K as Field), N as Field, "Invalid path length");

        let mut path = [0; K];
        let mut current_index = leaf_index;
        let mut subtree_width = N;

        let mut current_sibling_index = sibling_index(current_index);

        path[0] = self.leaves[current_sibling_index];

        let mut subtree_offset: u32 = 0;

        for i in 1..K {
            current_index = current_index / 2;
            subtree_width = subtree_width / 2;

            current_sibling_index = sibling_index(current_index);

            path[i] = self.nodes[subtree_offset + current_sibling_index];

            subtree_offset += subtree_width;
        }

        path
    }
}

pub fn sibling_index(index: u32) -> u32 {
    if index % 2 == 0 {
        index + 1
    } else {
        index - 1
    }
}
