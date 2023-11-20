use crate::abis::{
    function_data::FunctionData,
    function_selector::FunctionSelector,
    membership_witness::FunctionLeafMembershipWitness,
};

struct ContractFunction {
    data: FunctionData,
    vk_hash: Field,
    acir_hash: Field,
    membership_witness: FunctionLeafMembershipWitness,
}

global default_constructor = ContractFunction {
    data: FunctionData {
        selector: FunctionSelector { inner: 999 },
        is_internal: false,
        is_private: true,
        is_constructor: true,
    },
    vk_hash: 0,
    acir_hash: 43214321,
    membership_witness: dep::std::unsafe::zeroed(),
};


// sibling_path taken from __snapshots__/noir_test_gen.test.ts.snap
global default_contract_function = ContractFunction {
    data: FunctionData {
        selector: FunctionSelector { inner: 1 },
        is_internal: false,
        is_private: true,
        is_constructor: false,
    },
    vk_hash: 0,
    acir_hash: 12341234,
    membership_witness: FunctionLeafMembershipWitness {
        leaf_index: 0,
        sibling_path: [
            0x1f2e3193c7187347a099ee7cb5d6ac077da6b18706fe5508e658a3d0a05494f7,
            0x2350c7d2f19119502d073142ff019af884d6513f38b92d5119b4999502282247,
            0x0837a67313f4dbbd8d6971c0672f961f0a3b9e218c1395d327915209292acbbf,
            0x2e0ef36ddc5db29acb6ef904999046f835ce7c78a40c3f7a0edb03b2f917a765,
        ],
    },
};

global default_public_function = ContractFunction {
    data: FunctionData {
        selector: FunctionSelector { inner: 2 },
        is_internal: false,
        is_private: false,
        is_constructor: false,
    },
    vk_hash: 0,
    acir_hash: 43214321,
    membership_witness: FunctionLeafMembershipWitness {
        leaf_index: 0,
        sibling_path: [
            0x1f2e3193c7187347a099ee7cb5d6ac077da6b18706fe5508e658a3d0a05494f7,
            0x2350c7d2f19119502d073142ff019af884d6513f38b92d5119b4999502282247,
            0x0837a67313f4dbbd8d6971c0672f961f0a3b9e218c1395d327915209292acbbf,
            0x2e0ef36ddc5db29acb6ef904999046f835ce7c78a40c3f7a0edb03b2f917a765,
        ],
    },
};
