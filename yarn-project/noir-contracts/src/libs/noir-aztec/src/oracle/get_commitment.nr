// Oracle function to get a commitment, its sibling path and index, without getting its preimage.

// commitment + index + root;
global COMMITMENT_GETTER_LENGTH = 3;

#[oracle(getCommitment)]
fn get_commitment_oracle(_commitment: Field) -> [Field; COMMITMENT_GETTER_LENGTH] {}

unconstrained fn get_commitment(commitment: Field) -> [Field; COMMITMENT_GETTER_LENGTH] {
    get_commitment_oracle(commitment)
}
