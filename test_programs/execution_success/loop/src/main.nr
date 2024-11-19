// Tests a very simple program.
//
// The features being tested is basic looping.
fn main(six_as_u32: u32) {
    assert_eq(loop(4), six_as_u32);
    assert_eq(loop_incl(3), six_as_u32);
    assert(plain_loop() == six_as_u32);
}

fn loop(x: u32) -> u32 {
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
