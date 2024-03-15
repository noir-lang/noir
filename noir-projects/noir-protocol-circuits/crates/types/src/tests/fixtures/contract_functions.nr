use crate::abis::{
    function_data::FunctionData, function_selector::FunctionSelector,
    membership_witness::FunctionLeafMembershipWitness
};

struct ContractFunction {
    data: FunctionData,
    vk_hash: Field,
    acir_hash: Field,
    membership_witness: FunctionLeafMembershipWitness,
}

// sibling_path taken from __snapshots__/noir_test_gen.test.ts.snap
global default_private_function = ContractFunction {
    data: FunctionData {
        selector: FunctionSelector { inner: 1010101 },
        is_private: true,
    },
    vk_hash: 0,
    acir_hash: 1111,
    membership_witness: FunctionLeafMembershipWitness {
        leaf_index: 0,
        sibling_path: [
            0x1e5cebe7a50c5c8fd1ebe19ed6bbf80f77819b12a2a28f334e895501e1cda574,
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
    membership_witness: FunctionLeafMembershipWitness {
        leaf_index: 2,
        sibling_path: [
            0x2d72ef5ebb7c974e1f5a8bed092f1cf1bf0a0cb1eda28516221ca7e5811ecf15,
            0x1bed44f12632c0a6343cd886bd3e548bb5e8d2fd35fe9bc81f28defd4ed885b0,
            0x0837a67313f4dbbd8d6971c0672f961f0a3b9e218c1395d327915209292acbbf,
            0x2e0ef36ddc5db29acb6ef904999046f835ce7c78a40c3f7a0edb03b2f917a765,
            0x1a9fdb505152f9c2baaffe4a30ee80775b58ebf8c2dde76435835b085c6f70ca,
        ],
    },
};

