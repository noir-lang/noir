use crate::{hash::accumulate_sha256, merkle_tree::merkle_tree::MerkleTree};

pub fn generate_full_sha_tree<let N: u32>(leaves: [Field; N]) -> MerkleTree<N> {
    let mut nodes = [0; N];
    let total_nodes = N - 1;
    let half_size = N / 2;
    // hash base layer
    for i in 0..half_size {
        nodes[i] = accumulate_sha256([leaves[2 * i], leaves[2 * i + 1]]);
    }
    // hash the other layers
    for i in 0..(total_nodes - half_size) {
        nodes[half_size + i] = accumulate_sha256([nodes[2 * i], nodes[2 * i + 1]]);
    }
    MerkleTree { leaves, nodes }
}
