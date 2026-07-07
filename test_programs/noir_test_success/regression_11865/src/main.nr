// The tests below return vectors from oracles as a `(semantic_length, contents)` pair;
// this works because in SSA this is the same, but normally we would just write `.returns(@[1, 2, 3])`.
use std::test::OracleMock;

#[oracle(int_vector_oracle)]
unconstrained fn int_vector_oracle() -> [(u32, u32)] {}

// Unit is zero-sized, so we cannot divide with an element size to figure
// out how long the returned vector is.
#[oracle(unit_vector_oracle)]
unconstrained fn unit_vector_oracle() -> [()] {}

#[test]
unconstrained fn use_content_length_when_semantic_is_zero() {
    let mut mock = OracleMock::mock("int_vector_oracle");
    // If the semantic length is zero, and we can calculate the length from
    // the element size, we assume the oracle didn't set it, because this
    // field used to be ignored. We will just use the one we can deduce.
    mock = mock.returns((0, [(1, 2), (3, 4)]));
    let res = int_vector_oracle();
    assert_eq(res.len(), 2);
    assert_eq(res, @[(1, 2), (3, 4)]);
}

#[test]
unconstrained fn accept_semantic_length_when_matches_content() {
    let mut mock = OracleMock::mock("int_vector_oracle");
    // This is the good case: the oracle has been updated to return the
    // correct semantic length, matching our calculation based on the
    // element size.
    mock = mock.returns((2, [(1, 2), (3, 4)]));
    let res = int_vector_oracle();
    assert_eq(res.len(), 2);
    assert_eq(res, @[(1, 2), (3, 4)]);
}

#[test(should_fail_with = "semantic length")]
unconstrained fn reject_semantic_length_when_does_not_match_content() {
    let mut mock = OracleMock::mock("int_vector_oracle");
    // The oracle returned an incorrect semantic length, so it has been
    // updated to return a value, but it's not expected: either data is
    // missing, or it looks like we would need to truncate it.
    // There shouldn't be a need to return more or less data from an
    // oracle than the exact semantic length, so reject this result.
    mock = mock.returns((4, [(1, 2), (3, 4)]));
    let _ = int_vector_oracle();
}

#[test]
unconstrained fn handle_return_of_empty_unit_vector() {
    let mut mock = OracleMock::mock("unit_vector_oracle");
    let empty: [(); 0] = [];
    // The oracle returns an zero elements for a zero sized types,
    // which is clearly a zero length array.
    mock = mock.returns((0, empty));
    let res = unit_vector_oracle();
    assert_eq(res.len(), 0);
    assert_eq(res, @[]);
}

#[test]
unconstrained fn handle_return_of_non_empty_unit_vector() {
    let mut mock = OracleMock::mock("unit_vector_oracle");
    let non_empty = [(), (), ()];
    // This is a zero length type, so we can only go by the semantic length:
    // even though it looks like a non-empty array is returned, it won't
    // take up physical space.
    mock = mock.returns((3, non_empty));
    let res = unit_vector_oracle();
    assert_eq(res.len(), 3);
    assert_eq(res, @[(), (), ()]);
}

#[test]
unconstrained fn handle_return_of_mismatched_unit_vector() {
    let mut mock = OracleMock::mock("unit_vector_oracle");
    let empty: [(); 0] = [];
    // This test proves that it doesn't matter if we return a different length
    // of zero-sized data: it's all empty, and only the semantic length matters.
    mock = mock.returns((3, empty));
    let res = unit_vector_oracle();
    assert_eq(res.len(), 3);
    assert_eq(res, @[(), (), ()]);
}

#[test(should_fail_with = "non-empty data for zero vector element size")]
unconstrained fn reject_non_empty_data_for_zero_element_length() {
    let mut mock = OracleMock::mock("unit_vector_oracle");
    // Here the oracle returns non-empty data for a zero type, which should be rejected.
    mock = mock.returns((0, [1, 2, 3]));
    let _ = unit_vector_oracle();
}
