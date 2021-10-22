// Returns one if the leaf is in the tree 
// and it is at the given index 
// and the hashpath proves this
// Currently we assume that it is a binary tree, so depth k implies a width of 2^k
// XXX: In the future we can add an arity parameter
#[foreign(merkle_membership)]
fn check_membership(_root : Field, _leaf : Field, _depth : Field) -> Field {}
