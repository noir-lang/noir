#[oracle(createCommitment)]
fn create_commitment_oracle(_commitment: Field) -> Field {}

unconstrained pub fn create_commitment(commitment: Field) {
    assert(create_commitment_oracle(commitment) == 0);
}
