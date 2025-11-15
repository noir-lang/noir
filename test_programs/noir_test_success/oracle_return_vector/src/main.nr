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

#[oracle(void_to_vectors)]
unconstrained fn void_to_vectors_oracle() -> ([Field], [Field]) {}

unconstrained fn void_to_vectors() -> ([Field], [Field]) {
    void_to_vectors_oracle()
}

// Test returning [array, vector].
// This should be fine, the array is allocated first, the vector is written to free memory.
#[test]
unconstrained fn test_void_to_array_and_vector_memory_does_not_overlap() {
    let a = [1, 2, 3];
    let v = &[4, 5];
    let _ = OracleMock::mock("void_to_array_and_vector").returns((a, v));
    let res = void_to_array_and_vector();
    assert_eq(res.0, a);
    assert_eq(res.1, v);
}

// Test returning [vector, array].
// The array is allocated after the vector; the array data must not overwrite the vector.
#[test]
unconstrained fn test_void_to_vector_and_array_memory_does_not_overlap() {
    let v = &[1, 2];
    let a = [3, 4, 5];
    let _ = OracleMock::mock("void_to_vector_and_array").returns((v, a));
    let res = void_to_vector_and_array();
    assert_eq(res.0, v);
    assert_eq(res.1, a);
}

// Test returning [vector, vector].
// Multiple vectors must not overwrite each others data.
#[test]
unconstrained fn test_void_to_vectors_memory_does_not_overlap() {
    let v1 = &[1, 2];
    let v2 = &[3, 4, 5];
    let _ = OracleMock::mock("void_to_vectors").returns((v1, v2));
    let res = void_to_vectors();
    assert_eq(res.0, v1);
    assert_eq(res.1, v2);
}

// Test making a multiple calls so that we have to allocate an array after
// previous calls have returned vectors, to check that the free memory pointer is
// updated in a way visible to later allocations.
#[test]
unconstrained fn test_multiple_calls_memory_does_not_overlap() {
    let v1 = &[1, 2];
    let a1 = [3, 4, 5];
    let v2 = &[6, 7];
    let a2 = [8, 9, 10];
    let _ = OracleMock::mock("void_to_vector_and_array").returns((v1, a1));
    let _ = OracleMock::mock("void_to_array_and_vector").returns((a2, v2));
    let res1 = void_to_vector_and_array();
    let res2 = void_to_array_and_vector();
    assert_eq(res1.0, v1);
    assert_eq(res1.1, a1);
    assert_eq(res2.0, a2);
    assert_eq(res2.1, v2);
}
