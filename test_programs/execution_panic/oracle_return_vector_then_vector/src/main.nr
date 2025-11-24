// Currently the AVM doesn't offer a way to correctly handle the return of multiple vectors from foreign calls,
// so we decided that the Brillig VM cannot do so either. This test is to show that trying to compile such code fails.

#[oracle(void_to_vectors)]
unconstrained fn void_to_vectors_oracle() -> ([Field], [Field]) {}

unconstrained fn void_to_vectors() -> ([Field], [Field]) {
    void_to_vectors_oracle()
}

unconstrained fn main() {
    let _ = void_to_vectors();
}
