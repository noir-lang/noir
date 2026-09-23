// Both conversions in a round trip preserve the original array's buffer source.
unconstrained fn main(a: [u8; 2], c: [u8; 2], n: u32) {
    let mut a = a;
    let mut c = c;
    for _i in 0..n {
        let previous = a;
        a = c.as_str_unchecked().as_bytes();
        c = previous.as_str_unchecked().as_bytes();
    }
    c[0] = c[0] + 1;
    assert_eq(a, [10, 20]);
    assert_eq(c, [2, 2]);
}
