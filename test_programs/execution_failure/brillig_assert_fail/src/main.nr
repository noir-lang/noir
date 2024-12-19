// Tests a very simple program.
// 
// The features being tested is using assert on brillig
fn main(x: Field) {
    assert(
        1 == unsafe {
            //@safety: testing context
            conditional(x as bool)
        }
    );
}

unconstrained fn conditional(x: bool) -> Field {
    assert(x);
    1
}
