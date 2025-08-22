// Tests a very simple program.
//
// The features being tested is basic looping.
fn main(six_as_u32: u32) {
    assert_eq(loop_excl(4), six_as_u32);
    assert_eq(loop_incl(3), six_as_u32);
    assert(plain_loop() == six_as_u32);
    assert(never_loop() == 0);

    // Safety: testing context
    unsafe {
        assert(basic_break() == true)
    }
}

fn loop_excl(x: u32) -> u32 {
    let mut sum = 0;
    for i in 0..x {
        sum = sum + i;
    }
    sum
}

fn loop_incl(x: u32) -> u32 {
    let mut sum = 0;
    for i in 0..=x {
        sum = sum + i;
    }
    sum
}

fn plain_loop() -> u32 {
    let mut sum = 0;
    for i in 0..4 {
        sum = sum + i;
    }
    sum
}

fn never_loop() -> u32 {
    let mut sum = 0;
    for i in 4..0 {
        sum = sum + i;
    }
    sum
}

unconstrained fn basic_break() -> bool {
    for idx_e in 0..5 {
        if (idx_e < 5) { break; };
    }
    true
}
