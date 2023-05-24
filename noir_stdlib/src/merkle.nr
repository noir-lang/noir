// Regular merkle tree means a append-only merkle tree (Explain why this is the only way to have privacy and alternatives if you don't want it)

// Returns one if the leaf is in the tree 
// and it is at the given index 
// and the hashpath proves this
// Currently we assume that it is a binary tree, so depth k implies a width of 2^k
// XXX: In the future we can add an arity parameter
fn check_membership(_root : Field, _leaf : Field, _index : Field, _hash_path: [Field]) -> Field {
    (compute_merkle_root(_leaf, _index, _hash_path) == _root) as Field
}

// Returns the root of the tree from the provided leaf and its hashpath, using pedersen hash
fn compute_merkle_root(leaf: Field, index: Field, hash_path: [Field]) -> Field {
    let n = hash_path.len();
    let index_bits = index.to_le_bits(n as u32);
    let mut current = leaf;
    for i in 0..n {
        let path_bit = index_bits[i] as bool;
        let (hash_left, hash_right) = if path_bit {
            (hash_path[i], current)
         } else {
            (current, hash_path[i])
         };

      current = crate::hash::pedersen([hash_left, hash_right])[0];
    };
    current
}

// Deprecated function.
// Previously this was the fallback implementation of `compute_merkle_root` however the Noir
// implementation is more efficient than the black box version so we removed it.
fn compute_root_from_leaf(leaf: Field, index: Field, hash_path: [Field]) -> Field {
   compute_merkle_root(leaf, index, hash_path)
}
