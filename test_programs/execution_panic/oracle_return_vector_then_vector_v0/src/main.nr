// With Brillig VM v0, trying to return multiple vectors causes a panic.

#[oracle(void_to_vectors)]
unconstrained fn void_to_vectors_oracle() -> ([Field], [Field]) {}

unconstrained fn void_to_vectors() -> ([Field], [Field]) {
    void_to_vectors_oracle()
}

unconstrained fn main() {
    let _ = void_to_vectors();
}
