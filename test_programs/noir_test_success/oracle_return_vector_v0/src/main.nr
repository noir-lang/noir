// With Brillig VM v0, we can return only return 1 vector from a foreign call,
// and it cannot be followed by arrays.
use std::test::OracleMock;

#[oracle(void_to_array_and_vector)]
unconstrained fn void_to_array_and_vector_oracle() -> ([Field; 3], [Field]) {}

unconstrained fn void_to_array_and_vector() -> ([Field; 3], [Field]) {
    void_to_array_and_vector_oracle()
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
