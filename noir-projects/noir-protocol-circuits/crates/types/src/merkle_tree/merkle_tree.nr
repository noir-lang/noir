struct MerkleTree<N> {
    leaves: [Field; N],
    nodes: [Field; N],
}

impl<N> MerkleTree<N> {
    pub fn new(leaves: [Field; N]) -> Self {
        let mut nodes = [0; N];

        // We need one less node than leaves, but we cannot have computed array lengths
        let total_nodes = N - 1;
        let half_size = N / 2;

        // hash base layer
        for i in 0..half_size {
            nodes[i] = dep::std::hash::pedersen_hash([leaves[2*i], leaves[2*i+1]]);
        }

        // hash the other layers
        for i in 0..(total_nodes - half_size) {
            nodes[half_size+i] = dep::std::hash::pedersen_hash([nodes[2*i], nodes[2*i+1]]);
        }

        MerkleTree { leaves, nodes }
    }

    fn get_root(self) -> Field {
        self.nodes[N - 2]
    }
}

