use dep::protocol_types::constants::{
    PUBLIC_DATA_TREE_HEIGHT,
    GENERATOR_INDEX__PUBLIC_LEAF_INDEX,
};
use dep::std::merkle::compute_merkle_root;

use crate::{
    context::PrivateContext,
    hash::pedersen_hash,
    oracle::get_sibling_path::get_sibling_path,
};

pub fn prove_public_value_inclusion(
    value: Field, // The value that we want to prove is in the public data tree
    storage_slot: Field, // The storage slot in which the value is stored
    block_number: u32, // The block at which we'll prove that the note exists
    context: PrivateContext
) {
    // 1) Get block header from oracle and ensure that the block hash is included in the archive.
    let block_header = context.get_block_header(block_number);

    // 2) Compute the public value leaf index.
    // We have to compute the leaf index here because unlike in the case of note commitments, public values are
    // not siloed with contract address so an oracle could cheat and give us a membership witness for arbitrary
    // value in the public data tree.
    let value_leaf_index = pedersen_hash(
        [context.this_address(), storage_slot],
        GENERATOR_INDEX__PUBLIC_LEAF_INDEX
    );

    // 3) Get the sibling path of the value leaf index in the public data tree at block `block_number`.
    let public_data_tree_id = 3; // TODO(#3443)
    let path: [Field; PUBLIC_DATA_TREE_HEIGHT] =
        get_sibling_path(block_number, public_data_tree_id, value_leaf_index);

    // 4) Prove that the value provided on input is in the public data tree at the given storage slot.
    assert(
        block_header.public_data_tree_root == compute_merkle_root(value, value_leaf_index, path),
        "Proving public value inclusion failed"
    );

    // --> Now we have traversed the trees all the way up to archive root and that way verified that a specific
    // `value` was really set in a given contract storage slot at block `block_number` in public data tree.
}