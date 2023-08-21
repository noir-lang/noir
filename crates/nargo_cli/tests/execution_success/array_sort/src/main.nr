fn main(xs : [u8; 3]) {
    let sorted = xs.sort();
    assert_eq(sorted[0], 1);
    assert_eq(sorted[1], 2);
    assert_eq(sorted[2], 3);
}
