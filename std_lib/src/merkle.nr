#[foreign(merkle_root)]
fn compute_root(_input : []Witness) -> Witness {}

#[foreign(merkle_membership)]
fn check_membership(_root : Witness, _leaf : Witness, _index : Witness, _hashpath : []Witness) -> Witness {}
