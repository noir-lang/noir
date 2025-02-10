// Tests a simple loop where we expect loop invariant instructions
// to be hoisted to the loop's pre-header block.
fn main(x: u32, y: u32) {
    loop_(4, x, y);
    array_read_loop(4, x);
}

fn loop_(upper_bound: u32, x: u32, y: u32) {
    for _ in 0..upper_bound {
        let mut z = x * y;
        z = z * x;
        assert_eq(z, 12);
    }
}

fn array_read_loop(upper_bound: u32, x: u32) {
    let arr = [2; 5];
    for i in 0..upper_bound {
        for j in 0..upper_bound {
            for _ in 0..upper_bound {
                assert_eq(arr[i], x);
                assert_eq(arr[j], x);
            }
        }
    }
}
