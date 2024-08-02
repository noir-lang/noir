use crate::abis::{function_data::FunctionData, function_selector::FunctionSelector};
use crate::merkle_tree::membership::MembershipWitness;
use crate::constants::FUNCTION_TREE_HEIGHT;

struct ContractFunction {
    data: FunctionData,
    vk_hash: Field,
    acir_hash: Field,
    membership_witness: MembershipWitness<FUNCTION_TREE_HEIGHT>,
}

// sibling_path taken from __snapshots__/noir_test_gen.test.ts.snap
global default_private_function = ContractFunction {
    data: FunctionData {
        selector: FunctionSelector { inner: 1010101 },
        is_private: true,
    },
    vk_hash: 0,
    acir_hash: 1111,
    membership_witness: MembershipWitness {
        leaf_index: 0,
        sibling_path: [
            0x1eca11cd2abe68905eab86bc4f207cde29eb7c2b4c2c9b27e3d8c2f8b8234c87,
            0x21dbfd1d029bf447152fcf89e355c334610d1632436ba170f738107266a71550,
            0x0bcd1f91cf7bdd471d0a30c58c4706f3fdab3807a954b8f5b5e3bfec87d001bb,
            0x06e62084ee7b602fe9abc15632dda3269f56fb0c6e12519a2eb2ec897091919d,
            0x03c9e2e67178ac638746f068907e6677b4cc7a9592ef234ab6ab518f17efffa0,
        ],
    },
};

global default_public_function = ContractFunction {
    data: FunctionData {
        selector: FunctionSelector { inner: 3030303 },
        is_private: false,
    },
    vk_hash: 0,
    acir_hash: 3333,
    membership_witness: MembershipWitness {
        leaf_index: 2,
        sibling_path: [
            0x2d72ef5ebb7c974e1f5a8bed092f1cf1bf0a0cb1eda28516221ca7e5811ecf15,
            0x0d18bf69ba03cceb5b3ad320c0439559ae10eabd18dea07ea239c8eb2a96c3f8,
            0x0837a67313f4dbbd8d6971c0672f961f0a3b9e218c1395d327915209292acbbf,
            0x2e0ef36ddc5db29acb6ef904999046f835ce7c78a40c3f7a0edb03b2f917a765,
            0x1a9fdb505152f9c2baaffe4a30ee80775b58ebf8c2dde76435835b085c6f70ca,
        ],
    },
};

