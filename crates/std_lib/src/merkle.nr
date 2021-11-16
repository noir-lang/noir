// Regular merkle tree means a append-only merkle tree (Explain why this is the only way to have privacy and alternatives if you don't want it)

// Returns one if the leaf is in the tree 
// and it is at the given index 
// and the hashpath proves this
// Currently we assume that it is a binary tree, so depth k implies a width of 2^k
// XXX: In the future we can add an arity parameter
#[foreign(merkle_membership)]
fn check_membership(_root : Field, _leaf : Field, _depth : Field) -> Field {}

// This only works for regular merkle trees, not sparse ones
// For sparse ones, this would be update_membership
// Returns the new root of the tree, one must pass in the old root of the tree,
// the leaf to insert and the depth of the tree
// 
// this method will insert into the underlying database and then create a merkle tree membership proof
// and then return the new root
// The root is only here to check the underlying database. No constraints are applied to the root
#[foreign(insert_regular_merkle)]
fn insert(_root : Field, _leaf : Field, _depth : Field) -> Field {}

// This is for sparse merkle trees. It is an update function, the value of the leaf is the index
// that we should place the leaf into 
// this method will insert into the underlying database and then create a merkle tree membership proof
// and then return the new root
// The root is only here to check the underlying database. No constraints are applied to the root
#[foreign(update_sparse_merkle)]
fn update(_root : Field, _leaf : Field, _depth : Field) -> Field {}
