unconstrained fn main() {
    let verification_key: [Field; 114] = [0; 114];
    let proof: [Field; 94] = [0; 94];
    let public_inputs: [Field; 1] = [0];
    let key_hash: Field = 0;
    let proof_type: u32 = 0;

    // Calling std::verify_proof_with_type from an unconstrained (Brillig) context
    // should produce a compile error:
    // "Cannot call `std::verify_proof_with_type` in unconstrained context"
    std::verify_proof_with_type(verification_key, proof, public_inputs, key_hash, proof_type);
}
