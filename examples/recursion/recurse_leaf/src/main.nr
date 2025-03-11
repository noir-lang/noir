global ULTRA_HONK_VK_SIZE: u32 = 128;
global ULTRA_HONK_PROOF_SIZE: u32 = 459;
global NUM_NON_ACCUMULATOR_PUBLIC_INPUTS: u32 = 3;
global HONK_IDENTIFIER: u32 = 1;

fn main(
    verification_key: [Field; ULTRA_HONK_VK_SIZE],
    public_inputs: pub [Field; NUM_NON_ACCUMULATOR_PUBLIC_INPUTS],
    key_hash: Field,
    proof: [Field; ULTRA_HONK_PROOF_SIZE],
    num: u64,
) -> pub u64 {
    // verify sum so far was computed correctly
    std::verify_proof_with_type(
        verification_key,
        proof,
        public_inputs,
        key_hash,
        HONK_IDENTIFIER,
    );

    // Take output of previous proof and add another number to it.
    public_inputs[2] as u64 + num
}
