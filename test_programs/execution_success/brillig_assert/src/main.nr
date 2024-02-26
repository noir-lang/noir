// Tests a very simple program.
// 
// The features being tested is using assert on brillig
fn main(x: field) {
    assert(1 == conditional(x as bool));
}

unconstrained fn conditional(x: bool) -> field {
    assert(x, f"Expected x to be false but got {x}");
    assert_eq(x, true, f"Expected x to be false but got {x}");
    1
}
