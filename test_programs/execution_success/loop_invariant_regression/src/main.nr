// Tests a simple loop where we expect loop invariant instructions
// to be hoisted to the loop's pre-header block.
fn main(x: u32, y: u32) {
    loop(4, x, y);
}

fn loop(upper_bound: u32, x: u32, y: u32) {
    for _ in 0..upper_bound {
        let mut z = x * y;
        z = z * x;
        assert_eq(z, 12);
    }
}
