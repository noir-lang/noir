use std::test::OracleMock;

#[oracle(void_to_array_and_vector)]
unconstrained fn void_to_array_and_vector_oracle() -> ([Field; 3], [Field]) {}

unconstrained fn void_to_array_and_vector() -> ([Field; 3], [Field]) {
    void_to_array_and_vector_oracle()
}

#[oracle(void_to_vector_and_array)]
unconstrained fn void_to_vector_and_array_oracle() -> ([Field], [Field; 3]) {}

unconstrained fn void_to_vector_and_array() -> ([Field], [Field; 3]) {
    void_to_vector_and_array_oracle()
}

#[test]
unconstrained fn test_void_to_array_and_vector() {
    let a = [1, 2, 3];
    let v = &[4, 5];
    let _ = OracleMock::mock("void_to_array_and_vector").returns((a, v));
    let res = void_to_array_and_vector();
    assert_eq(res.0, a);
    assert_eq(res.1, v);
}

// This would fail, but currently it results in a panic to prevent it from happening.
// #[test]
unconstrained fn test_void_to_vector_and_array() {
    let v = &[1, 2];
    let a = [3, 4, 5];
    let _ = OracleMock::mock("void_to_vector_and_array").returns((v, a));
    let res = void_to_vector_and_array();
    println(res);
    assert_eq(res.0, v);
    assert_eq(res.1, a);
}
