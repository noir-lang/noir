// Returns the merkle root
#[foreign(merkle_root)]
fn compute_root(_input : []Field) -> Field {}

// Returns one if the leaf is in the tree 
// and it is at the given index 
// and the hashpath proves this
#[foreign(merkle_membership)]
fn check_membership(_root : Field, _leaf : Field, _index : Field, _hashpath : []Field) -> Field {}
