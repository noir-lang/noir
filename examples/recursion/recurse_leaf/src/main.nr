global NUM_NON_ACCUMULATOR_PUBLIC_INPUTS: u32 = 3;

use bb_proof_verification::{UltraHonkVerificationKey, UltraHonkZKProof, verify_honk_proof};

fn main(
    verification_key: UltraHonkVerificationKey,
    public_inputs: pub [Field; NUM_NON_ACCUMULATOR_PUBLIC_INPUTS],
    key_hash: Field,
    proof: UltraHonkZKProof,
    num: u64,
) -> pub u64 {
    // verify sum so far was computed correctly
    verify_honk_proof(verification_key, proof, public_inputs, key_hash);

    // Take output of previous proof and add another number to it.
    public_inputs[2] as u64 + num
}
