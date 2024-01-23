// Tests a very simple program.
// 
// The features being tested is using assert on brillig
fn main(x: Field) {
    assert(1 == conditional(x as bool));
}

unconstrained fn conditional(x: bool) -> Field {
    assert(x, "x is false");
    // TODO: this is not accurately printing the assert message
    // got `Failed to solve brillig function, reason: explicit trap hit in brillig`
    // expected `x is false`
    assert_eq(x, true, "x is blah blah");
    let z = x as u8 + 20;
    assert_eq(z, 25);
    1
}
