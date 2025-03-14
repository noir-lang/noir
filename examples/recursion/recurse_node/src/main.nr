global ULTRA_HONK_VK_SIZE: u32 = 128;
global ULTRA_HONK_PROOF_SIZE: u32 = 459; // Number of fields when recursion is enabled
global NUM_NON_ACCUMULATOR_PUBLIC_INPUTS: u32 = 4;
global HONK_IDENTIFIER: u32 = 1;


fn main(
    verification_key: [Field; ULTRA_HONK_VK_SIZE],
    public_inputs: pub [Field; NUM_NON_ACCUMULATOR_PUBLIC_INPUTS],
    key_hash: Field,
    proof: [Field; ULTRA_HONK_PROOF_SIZE]
) -> pub u64 {
    // verify sum was computed correctly
    std::verify_proof_with_type(
        verification_key,
        proof,
        public_inputs,
        key_hash,
        HONK_IDENTIFIER,
    );

    public_inputs[3] as u64
}
