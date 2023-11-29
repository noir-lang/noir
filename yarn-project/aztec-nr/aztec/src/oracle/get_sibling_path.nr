use crate::constants_gen::NOTE_HASH_TREE_HEIGHT;
use crate::utils::arr_copy_slice;

#[oracle(getSiblingPath)]
fn get_sibling_path_oracle<N>(_block_number: Field, _tree_id: Field, _leaf_index: Field) -> [Field; N] {}

unconstrained pub fn get_sibling_path<N>(block_number: Field, tree_id: Field, leaf_index: Field) -> [Field; N] {
    let value: [Field; N] = get_sibling_path_oracle(block_number, tree_id, leaf_index);
    value
}
