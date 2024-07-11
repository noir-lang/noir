use dep::types::{
    merkle_tree::MerkleTree, recursion::{verification_key::VerificationKey, proof::RecursiveProof},
    traits::Serialize
};
use crate::{
    parity_public_inputs::ParityPublicInputs, root::root_parity_input::RootParityInput,
    utils::sha256_merkle_tree::Sha256MerkleTree
};
use dep::std;

global NUM_BASE_PARITY_PER_ROOT_PARITY: u64 = 4;

struct RootParityInputs {
    children: [RootParityInput; NUM_BASE_PARITY_PER_ROOT_PARITY],
}

impl RootParityInputs {
    pub fn root_parity_circuit(self) -> ParityPublicInputs {
        self.verify_child_proofs();
        let vk_tree_root = self.children[0].public_inputs.vk_tree_root;
        for i in 0..NUM_BASE_PARITY_PER_ROOT_PARITY {
            assert(
                self.children[i].public_inputs.vk_tree_root == vk_tree_root, "Inconsistent vk tree roots across base parity circuits"
            );
        }

        let mut sha_roots = [0; NUM_BASE_PARITY_PER_ROOT_PARITY];
        let mut converted_roots = [0; NUM_BASE_PARITY_PER_ROOT_PARITY];
        for i in 0..NUM_BASE_PARITY_PER_ROOT_PARITY {
            sha_roots[i] = self.children[i].public_inputs.sha_root;
            converted_roots[i] = self.children[i].public_inputs.converted_root;
        }

        let sha_tree = Sha256MerkleTree::new(sha_roots);
        let pedersen_tree = MerkleTree::new(converted_roots);

        ParityPublicInputs { sha_root: sha_tree.get_root(), converted_root: pedersen_tree.get_root(), vk_tree_root }
    }

    fn verify_child_proofs(self) {
        for i in 0..NUM_BASE_PARITY_PER_ROOT_PARITY {
            assert(
                self.children[i].verification_key.hash == self.children[0].verification_key.hash, "Inconsistent vk hashes across base parity circuits"
            );
            assert(
                self.children[i].vk_path == self.children[0].vk_path, "Inconsistent vk paths across base parity circuits"
            );
            //TODO: Do we need to validate this following hash
            //assert(hash(self.children[i].verification_key) == self.children[i].verification_key.hash);
            self.children[i].verify();
            // TODO(#7410) we need the tube vk to reinstate this
            // self.children[i].validate_in_vk_tree();
        }
    }
}

mod tests {
    use crate::{
        parity_public_inputs::ParityPublicInputs,
        root::{root_parity_input::RootParityInput, root_parity_inputs::RootParityInputs}
    };
    use dep::types::recursion::{verification_key::VerificationKey, proof::RecursiveProof};
    use dep::types::tests::fixtures;
    use dep::types::constants::BASE_PARITY_INDEX;

    fn test_setup() -> [RootParityInput; 4] {
        // 31 byte test SHA roots
        let children_sha_roots = [
            0xb3a3fc1968999f2c2d798b900bdf0de41311be2a4d20496a7e792a521fc8ab,
            0x43f78e0ebc9633ce336a8c086064d898c32fb5d7d6011f5427459c0b8d14e9,
            0x024259b6404280addcc9319bc5a32c9a5d56af5c93b2f941fa326064fbe963,
            0x53042d820859d80c474d4694e03778f8dc0ac88fc1c3a97b4369c1096e904a
        ];

        let vk_tree = fixtures::vk_tree::get_vk_merkle_tree();

        let vk_hash = vk_tree.leaves[BASE_PARITY_INDEX];
        let vk_path = vk_tree.get_sibling_path(BASE_PARITY_INDEX);
        let vk_tree_root = vk_tree.get_root();

        let mut vk1 = VerificationKey::empty();
        vk1.hash = vk_hash;

        let children = [
            RootParityInput {
                proof: RecursiveProof::empty(),
                verification_key: vk1,
                vk_path,
                public_inputs: ParityPublicInputs { sha_root: children_sha_roots[0], converted_root: 0, vk_tree_root }
            },
            RootParityInput {
                proof: RecursiveProof::empty(),
                verification_key: vk1,
                vk_path,
                public_inputs: ParityPublicInputs { sha_root: children_sha_roots[1], converted_root: 0, vk_tree_root }
            },
            RootParityInput {
                proof: RecursiveProof::empty(),
                verification_key: vk1,
                vk_path,
                public_inputs: ParityPublicInputs { sha_root: children_sha_roots[2], converted_root: 0, vk_tree_root }
            },
            RootParityInput {
                proof: RecursiveProof::empty(),
                verification_key: vk1,
                vk_path,
                public_inputs: ParityPublicInputs { sha_root: children_sha_roots[3], converted_root: 0, vk_tree_root }
            }
        ];
        children
    }

    #[test]
    fn test_sha_root_matches_frontier_tree() {
        let children = test_setup();
        let root_parity_inputs = RootParityInputs { children };

        let public_inputs = root_parity_inputs.root_parity_circuit();

        // 31 byte truncated root hash
        let expected_sha_root = 0xa0c56543aa73140e5ca27231eee3107bd4e11d62164feb411d77c9d9b2da47;

        assert(public_inputs.sha_root == expected_sha_root, "sha root does not match");
    }

    #[test(should_fail_with = "Inconsistent vk hashes across base parity circuits")]
    fn test_asserts_incorrect_vk_hash_1() {
        let mut vk2 = VerificationKey::empty();
        vk2.hash = 0x53042d820859d80c474d4694e03778f8dc0ac88fc1c3a97b4369c1096e904a;
        let mut children = test_setup();
        children[1].verification_key = vk2;
        let root_parity_inputs = RootParityInputs { children };

        let public_inputs = root_parity_inputs.root_parity_circuit();

        // 31 byte truncated root hash
        let expected_sha_root = 0xa0c56543aa73140e5ca27231eee3107bd4e11d62164feb411d77c9d9b2da47;

        assert(public_inputs.sha_root == expected_sha_root, "sha root does not match");
    }

    #[test(should_fail_with = "Inconsistent vk hashes across base parity circuits")]
    fn test_asserts_incorrect_vk_hash_2() {
        let mut vk2 = VerificationKey::empty();
        vk2.hash = 0x53042d820859d80c474d4694e03778f8dc0ac88fc1c3a97b4369c1096e904a;
        let mut children = test_setup();
        children[2].verification_key = vk2;
        let root_parity_inputs = RootParityInputs { children };

        let public_inputs = root_parity_inputs.root_parity_circuit();

        // 31 byte truncated root hash
        let expected_sha_root = 0xa0c56543aa73140e5ca27231eee3107bd4e11d62164feb411d77c9d9b2da47;

        assert(public_inputs.sha_root == expected_sha_root, "sha root does not match");
    }

    #[test(should_fail_with = "Inconsistent vk hashes across base parity circuits")]
    fn test_asserts_incorrect_vk_hash_3() {
        let mut vk2 = VerificationKey::empty();
        vk2.hash = 0x53042d820859d80c474d4694e03778f8dc0ac88fc1c3a97b4369c1096e904a;
        let mut children = test_setup();
        children[3].verification_key = vk2;
        let root_parity_inputs = RootParityInputs { children };

        let public_inputs = root_parity_inputs.root_parity_circuit();

        // 31 byte truncated root hash
        let expected_sha_root = 0xa0c56543aa73140e5ca27231eee3107bd4e11d62164feb411d77c9d9b2da47;

        assert(public_inputs.sha_root == expected_sha_root, "sha root does not match");
    }
}
