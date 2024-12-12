#[test(should_fail_with = "first")]
fn test_assert_message_preserved_during_optimization(a: u64, b: u64, c: u64) {
    assert((a + b) * (a - b) == c, "first");
    assert((a - b) * (a + b) == c, "second");
    assert((a + b) * (a - b) == c, "third");
    assert((2 * a + b) * (a - b / 2) == c * c, "fourth");
}
