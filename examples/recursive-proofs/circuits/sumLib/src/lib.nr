pub fn secretSum(a: u64, b: u64) -> u64 {
    a + b
}

#[test]
fn test_sum() {
    assert(secretSum(1, 2) == 3);
    // Uncomment to make test fail
    // assert(secretSum(1, 1) == 3);
}
