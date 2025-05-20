use crate::abis::{function_data::FunctionData, function_selector::FunctionSelector};
use crate::constants::{CLIENT_IVC_VERIFICATION_KEY_LENGTH_IN_FIELDS, FUNCTION_TREE_HEIGHT};
use crate::merkle_tree::membership::MembershipWitness;

pub struct ContractFunction {
    pub data: FunctionData,
    pub vk_hash: Field,
    pub membership_witness: MembershipWitness<FUNCTION_TREE_HEIGHT>,
}

pub global default_vk: [Field; CLIENT_IVC_VERIFICATION_KEY_LENGTH_IN_FIELDS] =
    [0; CLIENT_IVC_VERIFICATION_KEY_LENGTH_IN_FIELDS];

// sibling_path copy-and-pasted from __snapshots__/noir_test_gen.test.ts.snap (defaultContract 1)
pub global default_private_function: ContractFunction = ContractFunction {
    data: FunctionData { selector: FunctionSelector { inner: 1010101 }, is_private: true },
    vk_hash: crate::hash::verification_key_hash(default_vk),
    membership_witness: MembershipWitness {
        leaf_index: 0,
        sibling_path: [
            0x13c663ec0bb5d328246394c6b9f33988be9c05f3aa6cbf68f7ec6f795bf1645b,
            0x233d10e0c959338e062b1c76fb81e17fcf61643da568eac372bb7feac8bfa357,
            0x0e1ce4f11f4d51a7d3136abbd625315fff3876d322e46ad8401da96f8e43a614,
            0x1a0ca5eecb1430479902264f04e557160a4666fb42bb8b283c6397e3d17ac937,
            0x2a6595890719fef7967c209488728aa5342438ba52058a3c770202f55acf6854,
        ],
    },
};

pub fn get_protocol_contract_function(contract_index: u32) -> ContractFunction {
    ContractFunction {
        data: FunctionData {
            selector: FunctionSelector { inner: 98989 + contract_index },
            is_private: true,
        },
        vk_hash: crate::hash::verification_key_hash(default_vk),
        membership_witness: MembershipWitness {
            leaf_index: contract_index as Field,
            sibling_path: [
                0x1498a78842ab5efb2ac65befa149fbcfbcdc88ae49b73673965154361df14f3f,
                0x062d9223c973936f12eacf94db1cfb9499e5aee67c864677e238402006c20d21,
                0x26d4cb8f72c0330bbebdb587bf896ce790ec7d091ee6aa3aabcf9bd60b2de307,
                0x1d8f5034624cbef3663cf6a73a04bf82d34379e7e65f2b7d29fce4d6987ca7c1,
                0x1758942724d8e39e5a0bc52b58cd80e5b6f26bedcc2a00d4b2f0dc6019fb1bf1,
            ],
        },
    }
}
