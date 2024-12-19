// Tests a very simple program.
//
// The features being tested is basic conditonal on brillig
fn main(x: Field) {
    unsafe {
        //@safety: testing context
        assert(4 == conditional(x == 1));
    }
}

unconstrained fn conditional(x: bool) -> Field {
    if x {
        4
    } else {
        5
    }
}
