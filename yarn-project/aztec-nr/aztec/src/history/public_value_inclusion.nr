use dep::protocol_types::{
    constants::GENERATOR_INDEX__PUBLIC_LEAF_INDEX,
    hash::pedersen_hash,
};
use dep::std::merkle::compute_merkle_root;

use crate::{
    context::PrivateContext,
    oracle::get_public_data_witness::{
        get_public_data_witness,
        PublicDataWitness,
    },
    utils::{
        full_field_less_than,
    },
};

pub fn prove_public_value_inclusion(
    value: Field, // The value that we want to prove is in the public data tree
    storage_slot: Field, // The storage slot in which the value is stored
    block_number: u32, // The block at which we'll prove that the note exists
    context: PrivateContext
) {
    // 1) Get block header from oracle and ensure that the block hash is included in the archive.
    let header = context.get_header_at(block_number);

    // 2) Compute the leaf slot by siloing the storage slot with our own address
    let public_value_leaf_slot = pedersen_hash(
        [context.this_address().to_field(), storage_slot],
        GENERATOR_INDEX__PUBLIC_LEAF_INDEX
    );

    // 3) Get the membership witness of the slot
    let witness = get_public_data_witness(block_number, public_value_leaf_slot);

    // 4) Check that the witness matches the corresponding public_value
    let preimage = witness.leaf_preimage;

    // Here we have two cases. Code based on same checks in `validate_public_data_reads` in `base_rollup_inputs`
    // 1. The value is the same as the one in the witness
    // 2. The value was never initialized and is zero
    let is_less_than_slot = full_field_less_than(preimage.slot, public_value_leaf_slot);
    let is_next_greater_than = full_field_less_than(public_value_leaf_slot, preimage.next_slot);
    let is_max = ((preimage.next_index == 0) & (preimage.next_slot == 0));
    let is_in_range = is_less_than_slot & (is_next_greater_than | is_max);

    if is_in_range {
        assert_eq(value, 0, "Non-existant public data leaf value is non-zero");
    } else {
        assert_eq(preimage.slot, public_value_leaf_slot, "Public data slot don't match witness");
        assert_eq(preimage.value, value, "Public value does not match the witness");
    }

    // 5) Prove that the leaf we validated is in the public data tree
    assert(
        header.state.partial.public_data_tree.root
        == compute_merkle_root(preimage.hash(), witness.index, witness.path), "Proving public value inclusion failed"
    );
    // --> Now we have traversed the trees all the way up to archive root and that way verified that a specific
    // `value` was really set in a given contract storage slot at block `block_number` in public data tree.
}
