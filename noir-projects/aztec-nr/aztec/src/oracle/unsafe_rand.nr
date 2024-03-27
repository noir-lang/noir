#[oracle(getRandomField)]
fn rand_oracle() -> Field {}

// Called `unsafe_rand` because we do not constrain in circuit that we are dealing with an actual random value.
// Instead we just trust our PXE.
unconstrained pub fn unsafe_rand() -> Field {
    rand_oracle()
}
