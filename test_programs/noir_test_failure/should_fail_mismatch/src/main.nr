#[test(should_fail_with = "Not equal")]
fn test_different_string() {
    assert_eq(0, 1, "Different string");
}

// The failure reason is a substring of the expected message, but it should be the other way around
#[test(should_fail_with = "Definitely Not equal!")]
fn test_wrong_expectation() {
    assert_eq(0, 1, "Not equal");
}