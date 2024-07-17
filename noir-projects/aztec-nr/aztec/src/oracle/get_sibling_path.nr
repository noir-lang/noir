use dep::protocol_types::constants::PUBLIC_DATA_TREE_HEIGHT;

#[oracle(getSiblingPath)]
unconstrained fn get_sibling_path_oracle<let N: u32>(_block_number: u32, _tree_id: Field, _leaf_index: Field) -> [Field; N] {}

unconstrained pub fn get_sibling_path<let N: u32>(
    block_number: u32,
    tree_id: Field,
    leaf_index: Field
) -> [Field; N] {
    let value: [Field; N] = get_sibling_path_oracle(block_number, tree_id, leaf_index);
    value
}
