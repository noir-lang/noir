fn main(
    old_root: Field,
    old_leaf: Field,
    old_hash_path: [Field; 3],
    new_root: pub Field,
    leaf: Field,
    index: Field,
) {
    assert(old_root == compute_merkle_root(old_leaf, index, old_hash_path));

    let calculated_root = compute_merkle_root(leaf, index, old_hash_path);
    assert(new_root == calculated_root);
}

fn compute_merkle_root<let N: u32>(leaf: Field, index: Field, hash_path: [Field; N]) -> Field {
    let index_bits: [u1; N] = index.to_le_bits();
    let mut current = leaf;
    for i in 0..N {
        let path_bit = index_bits[i] != 0;
        let (hash_left, hash_right) = if path_bit {
            (hash_path[i], current)
        } else {
            (current, hash_path[i])
        };
        current = std::hash::pedersen_hash([hash_left, hash_right]);
    }
    current
}
