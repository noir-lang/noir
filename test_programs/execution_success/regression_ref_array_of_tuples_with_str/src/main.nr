// Regression test: assigning a reference to a tuple containing a string (array)
// into an array of such references inside a loop.
// Previously, the SSA gen incorrectly generated `array_get` on a reference
// when the reference's inner type contained an array (e.g. &mut [u8; 3]),
// because `contains_an_array()` returned true for references.
unconstrained fn main(x: pub bool) {
    let mut c: [&mut (bool, str<3>); 1] = [&mut (x, "AAA")];

    for _ in 0..3 {
        c[0] = &mut (false, "AYB");
    }

    let result = *c[0];
    assert_eq(result.0, false);
    assert_eq(result.1, "AYB");
}
