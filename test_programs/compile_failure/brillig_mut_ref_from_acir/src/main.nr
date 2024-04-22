unconstrained fn mut_ref_identity(value: &mut field) -> field {
    *value
}

fn main(mut x: field, y: pub field) {
    let returned_x = mut_ref_identity(&mut x);
    assert(returned_x == x);
}
