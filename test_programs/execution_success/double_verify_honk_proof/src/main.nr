// This circuit aggregates two Honk proof from `assert_statement`.
global SIZE_OF_PROOF_IF_LOGN_IS_28: u32 = 463;
global HONK_IDENTIFIER: u32 = 1;
fn main(
    verification_key: [Field; 128],
    // This is the proof without public inputs attached.
    // This means: the size of this does not change with the number of public inputs.
    proof: [Field; SIZE_OF_PROOF_IF_LOGN_IS_28],
    public_inputs: pub [Field; 1],
    key_hash: Field,
    // The second proof, currently set to be identical
    proof_b: [Field; SIZE_OF_PROOF_IF_LOGN_IS_28],
) {
    std::verify_proof_with_type(
        verification_key,
        proof,
        public_inputs,
        key_hash,
        HONK_IDENTIFIER,
    );
    std::verify_proof_with_type(
        verification_key,
        proof_b,
        public_inputs,
        key_hash,
        HONK_IDENTIFIER,
    );
}
