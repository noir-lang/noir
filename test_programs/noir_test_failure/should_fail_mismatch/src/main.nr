#[test(should_fail_with = "Not equal")]
fn test_different_string() {
    assert_eq(0, 1, "Different string");
}
// The assert message has a space
#[test(should_fail_with = "Not equal")]
fn test_with_extra_space() {
    assert_eq(0, 1, "Not equal ");
}
// The assert message has a space
#[test(should_fail_with = "Not equal")]
fn test_runtime_mismatch() {
    assert_eq(dep::std::hash::pedersen_commitment([27])[0], 0, "Not equal ");
}
