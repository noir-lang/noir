#[test(should_fail_with = "Not equal")]
fn test_should_fail_with_match() {
    assert_eq(0, 1, "Not equal");
}

#[test(should_fail)]
fn test_should_fail_without_match() {
    assert_eq(0, 1);
}

#[test(should_fail_with = "Not equal")]
fn test_should_fail_with_runtime_match() {
    assert_eq(dep::std::hash::pedersen_commitment([27]).x, 0, "Not equal");
}

#[test(should_fail)]
fn test_should_fail_without_runtime_match() {
    assert_eq(dep::std::hash::pedersen_commitment([27]).x, 0);
}
