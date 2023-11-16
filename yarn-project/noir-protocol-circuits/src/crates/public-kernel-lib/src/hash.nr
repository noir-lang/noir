use dep::types::address::Address;
use dep::aztec::{
    constants_gen,
    constants_gen::{GENERATOR_INDEX__PUBLIC_LEAF_INDEX},
    hash::sha256_to_field,
};

pub fn compute_public_data_tree_index(contract_address: Address, storage_slot: Field) -> Field {
    dep::std::hash::pedersen_hash_with_separator([
        contract_address.to_field(),
        storage_slot
    ], constants_gen::GENERATOR_INDEX__PUBLIC_LEAF_INDEX)
}

pub fn compute_public_data_tree_value(value: Field) -> Field {
    // as it's a public value, it doesn't require hashing.
    // leaving this function here in case we decide to change this.
    value
}