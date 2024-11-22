// This circuit aggregates two proofs from `assert_statement`.
fn main(
    verification_key: [Field; 114],
    // This is the proof without public inputs attached.
    //
    // This means: the size of this does not change with the number of public inputs.
    proof: [Field; 93],
    public_inputs: pub [Field; 1],
    // This is currently not public. It is fine given that the vk is a part of the circuit definition.
    // I believe we want to eventually make it public too though.
    key_hash: Field,
    proof_b: [Field; 93],
) {
    std::verify_proof(verification_key, proof, public_inputs, key_hash);

    std::verify_proof(verification_key, proof_b, public_inputs, key_hash);
}
