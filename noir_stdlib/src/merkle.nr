// Regular merkle tree means a append-only merkle tree (Explain why this is the only way to have privacy and alternatives if you don't want it)

// Returns one if the leaf is in the tree 
// and it is at the given index 
// and the hashpath proves this
// Currently we assume that it is a binary tree, so depth k implies a width of 2^k
// XXX: In the future we can add an arity parameter
#[foreign(merkle_membership)]
fn check_membership(_root : Field, _leaf : Field, _index : Field, _hash_path: [Field]) -> Field {}


#[alternative(merkle_membership)]
fn check_membership_in_noir(root : Field, leaf : Field, index : Field, hash_path: [Field]) -> Field {
    (compute_root_from_leaf(leaf, index, hash_path) == root) as Field
}

// Returns the root of the tree from the provided leaf and its hashpath, using pedersen hash
fn compute_root_from_leaf(leaf : Field, index : Field, hash_path: [Field]) -> Field {
    let n = crate::array::len(hash_path);
    let index_bits = crate::field::to_bits(index, n as u32);
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
