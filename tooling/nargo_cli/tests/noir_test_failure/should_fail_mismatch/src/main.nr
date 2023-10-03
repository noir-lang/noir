
#[test(should_fail_with = "Not equal")]
fn test_main_1() {
    assert_eq(0, 1, "Different string");
}

// The assert message has a space
#[test(should_fail_with = "Not equal")]
fn test_main_1() {
    assert_eq(0, 1, "Not equal ");
}