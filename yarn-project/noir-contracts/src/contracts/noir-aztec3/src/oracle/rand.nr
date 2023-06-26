

#[oracle(getRandomField)]
fn rand_oracle() -> [Field; 1] {}

unconstrained fn rand() -> Field {
    rand_oracle()[0]
}
