use dep::types::{
    constants::NUM_FIELDS_PER_SHA256,
    hash::accumulate_sha256,
};

// Note: Once we'll truncate sha256 to 1 Field we can nuke this and generalize the standard MerkleTree over different
// hash functions.
struct Sha256MerkleTree<N> {
    leaves: [[Field; NUM_FIELDS_PER_SHA256]; N],
    nodes: [[Field; NUM_FIELDS_PER_SHA256]; N],
}

impl<N> Sha256MerkleTree<N> {
    pub fn new(leaves: [[Field; NUM_FIELDS_PER_SHA256]; N]) -> Self {
        let mut nodes = [[0; NUM_FIELDS_PER_SHA256]; N];

        // We need one less node than leaves, but we cannot have computed array lengths
        let total_nodes = N - 1;
        let half_size = N / 2;

        // hash base layer
        for i in 0..half_size {
            nodes[i] = accumulate_sha256(
                [
                    U128::from_integer(leaves[2*i][0]),
                    U128::from_integer(leaves[2*i][1]),
                    U128::from_integer(leaves[2*i+1][0]),
                    U128::from_integer(leaves[2*i+1][1])
                ]
            );
        }

        // hash the other layers
        for i in 0..(total_nodes - half_size) {
            nodes[half_size+i] = accumulate_sha256(
                [
                    U128::from_integer(nodes[2*i][0]),
                    U128::from_integer(nodes[2*i][1]),
                    U128::from_integer(nodes[2*i+1][0]),
                    U128::from_integer(nodes[2*i+1][1])
                ]
            );
        }

        Sha256MerkleTree { leaves, nodes }
    }

    fn get_root(self) -> [Field; NUM_FIELDS_PER_SHA256] {
        self.nodes[N - 2]
    }
}