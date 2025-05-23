global HONK_PROOF_SIZE: u32 = 456;
global HONK_IDENTIFIER: u32 = 1;

fn main(
    verification_key: [Field; 112],
    proof: [Field; HONK_PROOF_SIZE],
    public_inputs: [Field; 2]
) {
    std::verify_proof_with_type(
        verification_key,
        proof,
        public_inputs,
        0x0,
        HONK_IDENTIFIER,
    );
}
