// Create a nullifier
#[oracle(createNullifier)]
fn create_nullifier_oracle(_nullifier: Field) -> Field {}

unconstrained fn create_nullifier(nullifier: Field) {
    assert(create_nullifier_oracle(nullifier) == 0);
}
