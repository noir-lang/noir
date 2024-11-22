// This circuit aggregates two recursive proofs from `double_verify_proof_recursive`.
// Recursive aggregation is a backend-specific process and it is expected for backends
// to attach any extra data they may need (e.g. aggregation objects) to their proofs.
// Whether the proof we are verifying itself contains a recursive proof is expected to be
// a circuit constant by the barretenberg. Barretenberg hides this circuit constant in the
// proof serialization. Thus, we must have separate circuits for verifying a normal proof and a recursive proof
// with two different proof sizes.
fn main(
    verification_key: [Field; 114],
    // This is the proof without user-specified public inputs attached.
    //
    // This means: the size of this does not change with the number of public inputs unless
    // they have been attached by the backend.
    proof: [Field; 109],
    public_inputs: pub [Field; 1],
    // This is currently not public. It is fine given that the vk is a part of the circuit definition.
    // I believe we want to eventually make it public too though.
    key_hash: Field,
    proof_b: [Field; 109],
) {
    std::verify_proof(verification_key, proof, public_inputs, key_hash);

    std::verify_proof(verification_key, proof_b, public_inputs, key_hash);
}
