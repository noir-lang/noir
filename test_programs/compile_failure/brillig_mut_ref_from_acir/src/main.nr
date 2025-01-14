unconstrained fn mut_ref_identity(value: &mut Field) -> Field {
    *value
}

fn main(mut x: Field, y: pub Field) {
    /// Safety: testing context
    let returned_x = unsafe { mut_ref_identity(&mut x) };
    assert(returned_x == x);
}
