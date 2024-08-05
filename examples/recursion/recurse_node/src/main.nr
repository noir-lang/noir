fn main(
    verification_key: [Field; 114],
    public_inputs: pub [Field; 4],
    key_hash: Field,
    proof: [Field; 109]
) -> pub u64 {
    // verify sum was computed correctly
    std::verify_proof(verification_key, proof, public_inputs, key_hash);
    public_inputs[3] as u64
}
