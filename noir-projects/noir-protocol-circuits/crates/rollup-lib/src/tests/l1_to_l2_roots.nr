use dep::types::constants::ROOT_PARITY_INDEX;
use dep::types::tests::fixtures;
use dep::parity_lib::{root::root_rollup_parity_input::RootRollupParityInput, ParityPublicInputs};

pub fn default_root_rollup_parity_input() -> RootRollupParityInput {
    let mut input = RootRollupParityInput::empty();

    let vk_index = ROOT_PARITY_INDEX;
    let vk_tree = fixtures::vk_tree::get_vk_merkle_tree();
    let vk_hash = vk_tree.leaves[vk_index];
    let vk_path = vk_tree.get_sibling_path(vk_index);
    let vk_tree_root = vk_tree.get_root();

    input.verification_key.hash = vk_hash;
    input.vk_path = vk_path;
    input.public_inputs.vk_tree_root = vk_tree_root;

    input
}
