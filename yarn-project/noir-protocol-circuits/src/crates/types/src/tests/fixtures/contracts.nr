use crate::abis::membership_witness::ContractLeafMembershipWitness;
use crate::address::{Address, EthAddress};
use crate::tests::fixtures;

struct ContractData {
    contract_address_salt: Field,
    address: Address,
    portal_contract_address: EthAddress,
    membership_witness: ContractLeafMembershipWitness,
    function_tree_root: Field,
}

global default_contract = ContractData {
    contract_address_salt: 34567,
    address: Address { inner: 12345 },
    portal_contract_address: EthAddress { inner: 23456 },
    membership_witness: ContractLeafMembershipWitness {
        leaf_index: 0,
        sibling_path: fixtures::contract_tree::SIBLING_PATHS[0],
    },
    function_tree_root: 0x2143c6a86ded93cd75de5680c57c61a2a178e0a840d8649a9e0411032f969f1d,
};