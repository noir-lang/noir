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
            0x233d10e0c959338e062b1c76fb81e17fcf61643da568eac372bb7feac8bfa357,
            0x0e1ce4f11f4d51a7d3136abbd625315fff3876d322e46ad8401da96f8e43a614,
            0x1a0ca5eecb1430479902264f04e557160a4666fb42bb8b283c6397e3d17ac937,
            0x2a6595890719fef7967c209488728aa5342438ba52058a3c770202f55acf6854,
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
            0x27b1d0839a5b23baf12a8d195b18ac288fcf401afb2f70b8a4b529ede5fa9fed,
            0x1204fec45a58f11be9981f153319860aea2ece9152efab01885006e53dd70a79,
            0x0e1ce4f11f4d51a7d3136abbd625315fff3876d322e46ad8401da96f8e43a614,
            0x1a0ca5eecb1430479902264f04e557160a4666fb42bb8b283c6397e3d17ac937,
            0x2a6595890719fef7967c209488728aa5342438ba52058a3c770202f55acf6854,
        ],
    },
};

