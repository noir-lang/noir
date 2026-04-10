unconstrained fn main(arr: [u32; 3], idx: u32) {
    let mut x: [u32; 3] = arr;
    let y: &mut [u32; 3] = &mut x;
    let z = x;
    let f1 = || {
        y[idx] = 100;
        assert_eq(y[idx], 100);
    };
    f1();
    assert_eq(z[idx], 1);
}
