use crate::abis::membership_witness::ContractLeafMembershipWitness;
use crate::address::{AztecAddress, EthAddress};
use crate::tests::fixtures;

struct ContractData {
    contract_address_salt: Field,
    address: AztecAddress,
    portal_contract_address: EthAddress,
    membership_witness: ContractLeafMembershipWitness,
    function_tree_root: Field,
}

global default_contract = ContractData {
    contract_address_salt: 34567,
    address: AztecAddress { inner: 12345 },
    portal_contract_address: EthAddress { inner: 23456 },
    membership_witness: ContractLeafMembershipWitness {
        leaf_index: 0,
        sibling_path: fixtures::contract_tree::SIBLING_PATHS[0],
    },
    function_tree_root: 0x1c55b3903b8b2812ba2a2315edb30aa9b9d9c99c153e9215cc01ec8979a8e9be,
};

global parent_contract = ContractData {
    contract_address_salt: 11111,
    address: AztecAddress { inner: 667788 },
    portal_contract_address: EthAddress { inner: 990011 },
    membership_witness: ContractLeafMembershipWitness {
        leaf_index: 1,
        sibling_path: fixtures::contract_tree::SIBLING_PATHS[1],
    },
    function_tree_root: 0x02b3f6b0a36bd01f08cee2d607dbe08894bb8c58159e87bb17db28cad43291d4,
};