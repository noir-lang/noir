// Regular merkle tree means a append-only merkle tree (Explain why this is the only way to have privacy and alternatives if you don't want it)
// Currently we assume that it is a binary tree, so depth k implies a width of 2^k
// XXX: In the future we can add an arity parameter
// Returns the merkle root of the tree from the provided leaf, its hashpath, using a pedersen hash function.
#[deprecated("This function will be removed from the stdlib in version 1.0.0-beta.4")]
pub fn compute_merkle_root<let N: u32>(leaf: Field, index: Field, hash_path: [Field; N]) -> Field {
    let index_bits: [u1; N] = index.to_le_bits();
    let mut current = leaf;
    for i in 0..N {
        let path_bit = index_bits[i] as bool;
        let (hash_left, hash_right) = if path_bit {
            (hash_path[i], current)
        } else {
            (current, hash_path[i])
        };
        current = crate::hash::pedersen_hash([hash_left, hash_right]);
    }
    current
}
