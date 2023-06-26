// Create a nullifier
#[oracle(createNullifier)]
fn create_nullifier_oracle(_nullifier: Field) -> [Field; 1] {}

unconstrained fn create_nullifier(nullifier: Field) {
    constrain create_nullifier_oracle(nullifier)[0] == 0;
}
