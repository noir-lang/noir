use crate::{parity_public_inputs::ParityPublicInputs, utils::sha256_merkle_tree::Sha256MerkleTree};
use dep::types::{constants::NUM_MSGS_PER_BASE_PARITY, merkle_tree::MerkleTree, utils::uint256::U256};

struct BaseParityInputs {
    msgs: [Field; NUM_MSGS_PER_BASE_PARITY],
    vk_tree_root: Field,
}

impl BaseParityInputs {
    pub fn base_parity_circuit(self) -> ParityPublicInputs {
        let sha_tree = Sha256MerkleTree::new(self.msgs);
        let pedersen_tree = MerkleTree::new(self.msgs);

        ParityPublicInputs {
            sha_root: sha_tree.get_root(),
            converted_root: pedersen_tree.get_root(),
            vk_tree_root: self.vk_tree_root
        }
    }
}

#[test]
fn test_sha_root_matches_frontier_tree() {
    // 31 byte msgs
    let msgs = [
        0x151de48ca3efbae39f180fe00b8f472ec9f25be10b4f283a87c6d783935370,
        0x14c2ea9dedf77698d4afe23bc663263eed0bf9aa3a8b17d9b74812f185610f,
        0x1570cc6641699e3ae87fa258d80a6d853f7b8ccb211dc244d017e2ca6530f8,
        0x2806c860af67e9cd50000378411b8c4c4db172ceb2daa862b259b689ccbdc1
    ];

    let base_parity_inputs = BaseParityInputs { msgs, vk_tree_root: 42 };
    let public_inputs = base_parity_inputs.base_parity_circuit();

    // 31 byte truncated root hash
    let expected_sha_root = 0xfc986d54a5e0af4f6e0d49399b9806c2b225e6c652fa5a831ecf6c6c29719d;

    assert(public_inputs.sha_root == expected_sha_root, "sha root does not match");
}
