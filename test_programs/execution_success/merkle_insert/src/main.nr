fn main(
    old_root: Field,
    old_leaf: Field,
    old_hash_path: [Field; 3],
    new_root: pub Field,
    leaf: Field,
    index: Field,
) {
    assert(old_root == std::merkle::compute_merkle_root(old_leaf, index, old_hash_path));

    let calculated_root = std::merkle::compute_merkle_root(leaf, index, old_hash_path);
    assert(new_root == calculated_root);
}
