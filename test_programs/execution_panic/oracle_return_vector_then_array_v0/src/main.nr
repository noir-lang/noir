// With Brillig VM v0, trying to return a vector then an array causes a panic.

#[oracle(void_to_vector_and_array)]
unconstrained fn void_to_vector_and_array_oracle() -> ([Field], [Field; 3]) {}

unconstrained fn void_to_vector_and_array() -> ([Field], [Field; 3]) {
    void_to_vector_and_array_oracle()
}

unconstrained fn main() {
    let _ = void_to_vector_and_array();
}
