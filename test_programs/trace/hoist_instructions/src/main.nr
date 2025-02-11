fn main(x: u32, y: u32) {
    loopfn(4, x, y);
}

fn loopfn(upper_bound: u32, x: u32, y: u32) {
    for _ in 0..upper_bound {
        let mut z = x * y;
        z = z * x;
        assert_eq(z, 12);
    }
}

