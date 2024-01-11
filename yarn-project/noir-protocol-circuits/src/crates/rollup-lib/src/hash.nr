use dep::types::{
    abis::global_variables::GlobalVariables,
    constants::GENERATOR_INDEX__BLOCK_HASH,
    state_reference::StateReference,
};

pub fn compute_block_hash_with_globals(globals: GlobalVariables, state: StateReference) -> Field {
    let inputs = [
        globals.hash(), state.partial.note_hash_tree.root, state.partial.nullifier_tree.root, state.partial.contract_tree.root, state.l1_to_l2_message_tree.root, state.partial.public_data_tree.root
    ];

    dep::std::hash::pedersen_hash_with_separator(inputs, GENERATOR_INDEX__BLOCK_HASH)
}
