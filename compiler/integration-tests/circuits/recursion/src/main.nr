global SIZE_OF_PROOF_IF_LOGN_IS_28: u32 = 456;
global HONK_IDENTIFIER: u32 = 1;

fn main(
    verification_key: [Field; 128],
    proof: [Field; SIZE_OF_PROOF_IF_LOGN_IS_28],
    public_inputs: [Field; 1],
    key_hash: Field
) {
    std::verify_proof_with_type(
        verification_key,
        proof,
        public_inputs,
        key_hash,
        HONK_IDENTIFIER,
    );
}
