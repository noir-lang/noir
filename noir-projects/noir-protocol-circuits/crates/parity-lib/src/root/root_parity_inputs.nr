use dep::types::{
    constants::NUM_FIELDS_PER_SHA256,
    merkle_tree::MerkleTree,
    mocked::AggregationObject,
};
use crate::{
    parity_public_inputs::ParityPublicInputs,
    root::root_parity_input::RootParityInput,
    utils::sha256_merkle_tree::Sha256MerkleTree,
};

global NUM_BASE_PARITY_PER_ROOT_PARITY: u64 = 4;

struct RootParityInputs {
    children: [RootParityInput; NUM_BASE_PARITY_PER_ROOT_PARITY],
}

impl RootParityInputs {
    pub fn root_parity_circuit(self) -> ParityPublicInputs {
        // TODO: verify proofs of inputs.children

        let mut sha_roots = [[0; NUM_FIELDS_PER_SHA256]; NUM_BASE_PARITY_PER_ROOT_PARITY];
        let mut converted_roots = [0; NUM_BASE_PARITY_PER_ROOT_PARITY];
        for i in 0..NUM_BASE_PARITY_PER_ROOT_PARITY {
            sha_roots[i] = self.children[i].public_inputs.sha_root;
            converted_roots[i] = self.children[i].public_inputs.converted_root;
        }

        let sha_tree = Sha256MerkleTree::new(sha_roots);
        let pedersen_tree = MerkleTree::new(converted_roots);

        ParityPublicInputs {
            aggregation_object: AggregationObject {},
            sha_root: sha_tree.get_root(),
            converted_root: pedersen_tree.get_root(),
        }
    }
}

mod tests {
    use crate::{
        parity_public_inputs::ParityPublicInputs,
        root::{
            root_parity_input::RootParityInput,
            root_parity_inputs::RootParityInputs,
        }
    };
    use dep::types::mocked::{AggregationObject, Proof};

    #[test]
    fn test_sha_root_matches_frontier_tree() {
        let children_sha_roots = [
            [0x00000000000000000000000000000000b3a3fc1968999f2c2d798b900bdf0de4, 0x000000000000000000000000000000001311be2a4d20496a7e792a521fc8abac],
            [0x0000000000000000000000000000000043f78e0ebc9633ce336a8c086064d898, 0x00000000000000000000000000000000c32fb5d7d6011f5427459c0b8d14e91f],
            [0x00000000000000000000000000000000024259b6404280addcc9319bc5a32c9a, 0x000000000000000000000000000000005d56af5c93b2f941fa326064fbe9636c],
            [0x0000000000000000000000000000000053042d820859d80c474d4694e03778f8, 0x00000000000000000000000000000000dc0ac88fc1c3a97b4369c1096e904ae7],
        ];

        let children = [
            RootParityInput { proof: Proof {}, public_inputs: ParityPublicInputs { aggregation_object: AggregationObject {}, sha_root: children_sha_roots[0], converted_root: 0 } },
            RootParityInput { proof: Proof {}, public_inputs: ParityPublicInputs { aggregation_object: AggregationObject {}, sha_root: children_sha_roots[1], converted_root: 0 } },
            RootParityInput { proof: Proof {}, public_inputs: ParityPublicInputs { aggregation_object: AggregationObject {}, sha_root: children_sha_roots[2], converted_root: 0 } },
            RootParityInput { proof: Proof {}, public_inputs: ParityPublicInputs { aggregation_object: AggregationObject {}, sha_root: children_sha_roots[3], converted_root: 0 } },
        ];

        let root_parity_inputs = RootParityInputs { children };

        let public_inputs = root_parity_inputs.root_parity_circuit();

        // 8e7d8bf0ef7ebd1607cc7ff9f2fbacf4574ee5b692a5a5ac1e7b1594067b9049 converted to 2 fields
        let expected_sha_root = [
            0x000000000000000000000000000000008e7d8bf0ef7ebd1607cc7ff9f2fbacf4,
            0x00000000000000000000000000000000574ee5b692a5a5ac1e7b1594067b9049
        ];

        assert(public_inputs.sha_root == expected_sha_root, "sha root does not match");
    }
}