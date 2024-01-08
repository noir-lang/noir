use crate::abis::global_variables::GlobalVariables;
use dep::types::constants::GENERATOR_INDEX__BLOCK_HASH;

pub fn compute_block_hash_with_globals(
    globals: GlobalVariables,
    note_hash_tree_root: Field,
    nullifier_tree_root: Field,
    contract_tree_root: Field,
    l1_to_l2_message_tree_root: Field,
    public_data_tree_root: Field
) -> Field {
    let inputs = [
        globals.hash(), note_hash_tree_root, nullifier_tree_root, contract_tree_root, l1_to_l2_message_tree_root, public_data_tree_root
    ];

    dep::std::hash::pedersen_hash_with_separator(inputs, GENERATOR_INDEX__BLOCK_HASH)
}
