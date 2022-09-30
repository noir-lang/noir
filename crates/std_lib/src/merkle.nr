// Regular merkle tree means a append-only merkle tree (Explain why this is the only way to have privacy and alternatives if you don't want it)

// Returns one if the leaf is in the tree 
// and it is at the given index 
// and the hashpath proves this
// Currently we assume that it is a binary tree, so depth k implies a width of 2^k
// XXX: In the future we can add an arity parameter
#[foreign(merkle_membership)]
fn check_membership(_root : Field, _leaf : Field, _index : Field, _hash_path: [Field]) -> Field {}


// Returns the root of the tree from the provided leaf and its hashpath, using pedersen hash
fn compute_root_from_leaf(leaf : Field, index : Field, hash_path: [Field;3]) -> Field {
    let n = 3;    //TODO: use array::len(hash_path); when monomorphisation PR is merged
    let index_bits = crate::to_bits(index, n as u32);
    let mut current = leaf;
    let n2 = 3;    //TODO: remove when PR #350 is merged
    for i in 0..n2 {
        let path_bit = index_bits[i] as bool;
        let path_elem = hash_path[i];
        
        let mut hash_left = 0;
        let mut hash_right = 0;
        if path_bit {
         hash_right = current;
         hash_left = path_elem;

         } else {
         hash_left = current;
         hash_right = path_elem;
         }
        //TODO use this when PR#353 is merged
        //  let (hash_left, hash_right) = if path_bit {
        //     (path_elem, current)
        //  } else {
        //     (current, path_elem)
        //  };

      current = crate::hash::pedersen([hash_left, hash_right])[0];
    };
    current
}
