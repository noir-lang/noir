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

struct InvalidPointError {
    point: dep::std::hash::PedersenPoint,
}

#[test(should_fail_with = "InvalidPointError { point: PedersenPoint { x: 0x1cea3a116d01eb94d568ef04c3dfbc39f96f015ed801ab8958e360d406503ce0, y: 0x2721b237df87234acc36a238b8f231a3d31d18fe3845fff4cc59f0bd873818f8 } }")]
fn test_should_fail_with_struct() {
    let hash = dep::std::hash::pedersen_commitment([27]);
    assert_eq(hash.x, 0, InvalidPointError { point: hash });
}

#[test(should_fail_with = "A: 0x00 is not 1!")]
fn test_should_fail_with_basic_type_fmt_string() {
    let a = 0;
    let b = 1;
    assert_eq(a, b, f"A: {a} is not 1!");
}

#[test(should_fail_with = "Invalid hash: PedersenPoint { x: 0x1cea3a116d01eb94d568ef04c3dfbc39f96f015ed801ab8958e360d406503ce0, y: 0x2721b237df87234acc36a238b8f231a3d31d18fe3845fff4cc59f0bd873818f8 }")]
fn test_should_fail_with_struct_fmt_string() {
    let hash = dep::std::hash::pedersen_commitment([27]);
    assert_eq(hash.x, 0, f"Invalid hash: {hash}");
}

// Also test unconstrained versions

#[test(should_fail_with = "Not equal")]
unconstrained fn unconstrained_test_should_fail_with_match() {
    assert_eq(0, 1, "Not equal");
}

#[test(should_fail)]
unconstrained fn unconstrained_test_should_fail_without_match() {
    assert_eq(0, 1);
}

#[test(should_fail_with = "Not equal")]
unconstrained fn unconstrained_test_should_fail_with_runtime_match() {
    assert_eq(dep::std::hash::pedersen_commitment([27]).x, 0, "Not equal");
}

#[test(should_fail)]
unconstrained fn unconstrained_test_should_fail_without_runtime_match() {
    assert_eq(dep::std::hash::pedersen_commitment([27]).x, 0);
}

#[test(should_fail_with = "InvalidPointError { point: PedersenPoint { x: 0x1cea3a116d01eb94d568ef04c3dfbc39f96f015ed801ab8958e360d406503ce0, y: 0x2721b237df87234acc36a238b8f231a3d31d18fe3845fff4cc59f0bd873818f8 } }")]
unconstrained fn unconstrained_test_should_fail_with_struct() {
    let hash = dep::std::hash::pedersen_commitment([27]);
    assert_eq(hash.x, 0, InvalidPointError { point: hash });
}

#[test(should_fail_with = "A: 0x00 is not 1!")]
unconstrained fn unconstrained_test_should_fail_with_basic_type_fmt_string() {
    let a = 0;
    let b = 1;
    assert_eq(a, b, f"A: {a} is not 1!");
}

#[test(should_fail_with = "Invalid hash: PedersenPoint { x: 0x1cea3a116d01eb94d568ef04c3dfbc39f96f015ed801ab8958e360d406503ce0, y: 0x2721b237df87234acc36a238b8f231a3d31d18fe3845fff4cc59f0bd873818f8 }")]
unconstrained fn unconstrained_test_should_fail_with_struct_fmt_string() {
    let hash = dep::std::hash::pedersen_commitment([27]);
    assert_eq(hash.x, 0, f"Invalid hash: {hash}");
}
