// With Brillig VM v0, we can return only return 1 vector from a foreign call,
// and it cannot be followed by arrays.
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

// Test returning [array, vector].
// This should be fine, the array is allocated first, the vector is written to free memory.
#[test]
unconstrained fn test_void_to_array_and_vector_memory_does_not_overlap() {
    let a = [1, 2, 3];
    let v = [4, 5].as_vector();
    let _ = OracleMock::mock("void_to_array_and_vector").returns((a, v));
    let res = void_to_array_and_vector();
    assert_eq(res.0, a);
    assert_eq(res.1, v);
}

// Test returning [vector, array].
// The array is allocated after the vector; the array data must not overwrite the vector.
#[test]
unconstrained fn test_void_to_vector_and_array_memory_does_not_overlap() {
    let v = [1, 2].as_vector();
    let a = [3, 4, 5];
    let _ = OracleMock::mock("void_to_vector_and_array").returns((v, a));
    let res = void_to_vector_and_array();
    assert_eq(res.0, v);
    assert_eq(res.1, a);
}
