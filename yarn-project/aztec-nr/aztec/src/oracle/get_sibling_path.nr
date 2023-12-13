use dep::protocol_types::constants::PUBLIC_DATA_TREE_HEIGHT;

#[oracle(getSiblingPath)]
fn get_sibling_path_oracle<N>(_block_number: u32, _tree_id: Field, _leaf_index: Field) -> [Field; N] {}

unconstrained pub fn get_sibling_path<N>(block_number: u32, tree_id: Field, leaf_index: Field) -> [Field; N] {
    let value: [Field; N] = get_sibling_path_oracle(block_number, tree_id, leaf_index);
    value
}

unconstrained pub fn get_public_data_sibling_path<N>(block_number: u32, leaf_index: Field) -> [Field; PUBLIC_DATA_TREE_HEIGHT] {
    let public_data_tree_id = 3; // TODO(#3443)
    get_sibling_path(block_number, public_data_tree_id, leaf_index)
}

// We don't implement specific function for other trees than public data tree because for the rest it makes sense
// to use get membership witness function instead.