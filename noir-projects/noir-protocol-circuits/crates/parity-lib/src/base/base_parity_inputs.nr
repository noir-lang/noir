use crate::{
    parity_public_inputs::ParityPublicInputs,
    utils::sha256_merkle_tree::Sha256MerkleTree,
};
use dep::types::{
    constants::{NUM_FIELDS_PER_SHA256, NUM_MSGS_PER_BASE_PARITY},
    merkle_tree::MerkleTree,
    mocked::AggregationObject,
    utils::uint256::U256,
};

struct BaseParityInputs {
    msgs: [Field; NUM_MSGS_PER_BASE_PARITY],
}

impl BaseParityInputs {
    pub fn base_parity_circuit(self) -> ParityPublicInputs {
        // TODO: nuke this hack once we truncate the sha256 in the frontier tree
        let mut converted_msgs = [[0; NUM_FIELDS_PER_SHA256]; NUM_MSGS_PER_BASE_PARITY];
        for i in 0..NUM_MSGS_PER_BASE_PARITY {
            let bytes = self.msgs[i].to_be_bytes(32);
            let mut result = [0; 32];
            for i in 0..32 {
                result[i] = bytes[i];
            }
            let msg_as_u256 = U256::from_bytes32(result);
            converted_msgs[i] = msg_as_u256.to_u128_limbs();
        }

        let sha_tree = Sha256MerkleTree::new(converted_msgs);
        let pedersen_tree = MerkleTree::new(self.msgs);

        ParityPublicInputs {
            aggregation_object: AggregationObject {},
            sha_root: sha_tree.get_root(),
            converted_root: pedersen_tree.get_root(),
        }
    }
}

#[test]
fn test_sha_root_matches_frontier_tree() {
    let msgs = [
      0x151de48ca3efbae39f180fe00b8f472ec9f25be10b4f283a87c6d78393537039,
      0x14c2ea9dedf77698d4afe23bc663263eed0bf9aa3a8b17d9b74812f185610f9e,
      0x1570cc6641699e3ae87fa258d80a6d853f7b8ccb211dc244d017e2ca6530f8a1,
      0x2806c860af67e9cd50000378411b8c4c4db172ceb2daa862b259b689ccbdc1e0
    ];

    let base_parity_inputs = BaseParityInputs { msgs };
    let public_inputs = base_parity_inputs.base_parity_circuit();

    // 0xb3a3fc1968999f2c2d798b900bdf0de41311be2a4d20496a7e792a521fc8abac converted to 2 fields
    let expected_sha_root = [
        0x00000000000000000000000000000000b3a3fc1968999f2c2d798b900bdf0de4,
        0x000000000000000000000000000000001311be2a4d20496a7e792a521fc8abac
    ];

    assert(public_inputs.sha_root == expected_sha_root, "sha root does not match");
}