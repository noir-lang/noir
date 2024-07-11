
// This circuit aggregates a single Honk proof from `assert_statement_recursive`.
global SIZE_OF_PROOF_IF_LOGN_IS_28 : u32 = 393;
fn main(
    verification_key: [Field; 103],
    // This is the proof without public inputs attached.
    // 
    // This means: the size of this does not change with the number of public inputs.
    proof: [Field; SIZE_OF_PROOF_IF_LOGN_IS_28],
    public_inputs: pub [Field; 1],
    // This is currently not public. It is fine given that the vk is a part of the circuit definition.
    // I believe we want to eventually make it public too though.
    key_hash: Field
) {
    std::verify_proof(
        verification_key.as_slice(),
        proof.as_slice(),
        public_inputs.as_slice(),
        key_hash
    );
}
