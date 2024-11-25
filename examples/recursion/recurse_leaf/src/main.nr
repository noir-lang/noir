fn main(
    verification_key: [Field; 114],
    public_inputs: pub [Field; 3],
    key_hash: Field,
    proof: [Field; 93],
    num: u64,
) -> pub u64 {
    // verify sum so far was computed correctly
    std::verify_proof(verification_key, proof, public_inputs, key_hash);
    // Take output of previous proof and add another number to it.
    public_inputs[2] as u64 + num
}
