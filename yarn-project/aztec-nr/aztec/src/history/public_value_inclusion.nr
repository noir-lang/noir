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
    let block_header = context.get_block_header(block_number);

    // 2) Compute the leaf slot by siloing the storage slot with our own address
    let public_value_leaf_slot = pedersen_hash(
        [context.this_address().to_field(), storage_slot],
        GENERATOR_INDEX__PUBLIC_LEAF_INDEX
    );

    // 3) Get the membership witness of the slot
    let witness = get_public_data_witness(block_number, public_value_leaf_slot);

    // 4) Check that the witness matches the corresponding public_value
    let preimage = witness.leaf_preimage;
    if preimage.slot == public_value_leaf_slot {
        assert_eq(preimage.value, value, "Public value does not match value in witness");
    } else {
        assert_eq(value, 0, "Got non-zero public value for non-existing slot");
        assert(full_field_less_than(preimage.slot, public_value_leaf_slot), "Invalid witness range");
        assert(full_field_less_than(public_value_leaf_slot, preimage.next_slot), "Invalid witness range");
    }

    // 5) Prove that the leaf we validated is in the public data tree
    assert(
        block_header.public_data_tree_root
        == compute_merkle_root(preimage.hash(), witness.index, witness.path), "Proving public value inclusion failed"
    );
    // --> Now we have traversed the trees all the way up to archive root and that way verified that a specific
    // `value` was really set in a given contract storage slot at block `block_number` in public data tree.
}
