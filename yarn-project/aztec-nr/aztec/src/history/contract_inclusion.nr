use dep::protocol_types::{
    abis::{
        complete_address::CompleteAddress,
        new_contract_data::NewContractData as ContractLeafPreimage,
    },
    address::{AztecAddress, EthAddress},
    point::Point,
};
use dep::std::merkle::compute_merkle_root;

use crate::{
    context::PrivateContext,
    oracle::get_membership_witness::get_contract_membership_witness,
};

// Proves that a contract exists at block `block_number` and returns its address.
// Note: This can be used to approximate a factory pattern --> a factory contract could perform this proof and that
//       way verify that a contract at a given address is what it expects. Then it could store it in an internal
//       map of contracts (like what Uniswap Factory does with pool contracts - it stores them in a mapping).
//       By passing in the construct hash the factory can also verify that the contract was constructed with the
//       correct constructor arguments. Typically the factory would store the expected construct hash and assert that
//       it is what it expects. The constructor param check is the reason of why we pass in the preimage of contract's
//       aztec address instead of just the address.
pub fn prove_contract_inclusion(
    deployer_public_key: Point,
    contract_address_salt: Field,
    function_tree_root: Field,
    constructor_hash: Field,
    portal_contract_address: EthAddress,
    block_number: u32, // The block at which we'll prove that the public value exists
    context: PrivateContext
) -> AztecAddress {
    // 1) Get block header from oracle and ensure that the block is included in the archive.
    let block_header = context.get_block_header(block_number);

    // 2) Compute the contract address
    let contract_address = CompleteAddress::compute(
        deployer_public_key,
        contract_address_salt,
        function_tree_root,
        constructor_hash
    ).address;

    // 3) Form the contract tree leaf preimage
    let preimage = ContractLeafPreimage { contract_address, portal_contract_address, function_tree_root };

    // 4) Get the contract tree leaf by hashing the preimage
    let contract_leaf = preimage.hash();

    // 5) Get the membership witness of the leaf in the contract tree
    let witness = get_contract_membership_witness(block_number, contract_leaf);

    // 6) Prove that the leaf is in the contract tree
    assert(
        block_header.contract_tree_root
        == compute_merkle_root(contract_leaf, witness.index, witness.path), "Proving contract inclusion failed"
    );

    // --> Now we have traversed the trees all the way up to archive root.

    contract_address
}
