use dep::types::{
    hash::accumulate_sha256,
};

// Note: Once we'll truncate sha256 to 1 Field we can nuke this and generalize the standard MerkleTree over different
// hash functions.
// TODO(Miranda): This is an interim version with 1 field sha256 - ideally we remove this and use accumulate_sha256 inside
// the general MT
struct Sha256MerkleTree<N> {
    leaves: [Field; N],
    nodes: [Field; N],
}

impl<N> Sha256MerkleTree<N> {
    pub fn new(leaves: [Field; N]) -> Self {
        let mut nodes = [0; N];

        // We need one less node than leaves, but we cannot have computed array lengths
        let total_nodes = N - 1;
        let half_size = N / 2;

        // hash base layer
        for i in 0..half_size {
            nodes[i] = accumulate_sha256(
                [
                    leaves[2*i],
                    leaves[2*i+1]
                ]
            );
        }

        // hash the other layers
        for i in 0..(total_nodes - half_size) {
            nodes[half_size+i] = accumulate_sha256(
                [
                    nodes[2*i],
                    nodes[2*i+1]
                ]
            );
        }

        Sha256MerkleTree { leaves, nodes }
    }

    fn get_root(self) -> Field {
        self.nodes[N - 2]
    }
}