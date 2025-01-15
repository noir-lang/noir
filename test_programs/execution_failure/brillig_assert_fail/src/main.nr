// Tests a very simple program.
//
// The features being tested is using assert on brillig
fn main(x: Field) {
    /// Safety: testing context
    assert(1 == unsafe { conditional(x as bool) });
}

unconstrained fn conditional(x: bool) -> Field {
    assert(x);
    1
}
