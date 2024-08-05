fn main(
    verification_key: [Field; 114],
    proof: [Field; 93],
    public_inputs: [Field; 1],
    key_hash: Field
) {
    std::verify_proof(verification_key, proof, public_inputs, key_hash)
}
