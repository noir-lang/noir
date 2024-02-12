#[oracle(getRandomField)]
fn rand_oracle() -> Field {}

unconstrained pub fn rand() -> Field {
    rand_oracle()
}
