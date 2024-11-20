// Tests a simple loop with a loop invariant
// that we expect to be hoisting to an ancestor block.
fn main(x: u32, y: u32) {
    loop(4, x, y);
}

fn loop(upper_bound: u32, x: u32, y: u32) {
    for _ in 0..upper_bound {
        let z = x * y;
        assert_eq(z, 6);
    }
}
