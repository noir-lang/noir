
fn main() {
    let unsorted: [u8; 3] = [3,1,2];
    let sorted = unsorted.sort();
    assert_eq(sorted[0], 1);
    assert_eq(sorted[1], 2);
    assert_eq(sorted[2], 3);
}
