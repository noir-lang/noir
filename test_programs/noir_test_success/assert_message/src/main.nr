#[test(should_fail_with = "first")]
fn test_assert_message_preserved_during_optimization(a: Field, b: Field, c: Field) {
    if (a + b) * (a - b) != c {
        assert((a + b) * (a - b) == c, "first");
        assert((a - b) * (a + b) == c, "second");
        assert((a + b) * (a - b) == c, "third");
        assert((2 * a + b) * (a - b / 2) == c * c, "fourth");
    } else {
        // The fuzzer might have generated a passing test.
        assert((a + b) * (a - b) != c, "first");
    }
}
