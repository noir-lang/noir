// Returns the merkle root
#[foreign(merkle_root)]
fn compute_root(_input : []Witness) -> Witness {}

// Returns one if the leaf is in the tree 
// and it is at the given index 
// and the hashpath proves this
#[foreign(merkle_membership)]
fn check_membership(_root : Witness, _leaf : Witness, _index : Witness, _hashpath : []Witness) -> Witness {}
