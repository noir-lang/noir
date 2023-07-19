

#[oracle(getRandomField)]
fn rand_oracle() -> Field {}

unconstrained fn rand() -> Field {
    rand_oracle()
}
