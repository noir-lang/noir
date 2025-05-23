global NUM_NON_ACCUMULATOR_PUBLIC_INPUTS: u32 = 3;

use bb_proof_verification::{verify_ultrahonk_proof, UltraHonkProof, UltraHonkVerificationKey};

fn main(
    verification_key: UltraHonkVerificationKey,
    public_inputs: pub [Field; NUM_NON_ACCUMULATOR_PUBLIC_INPUTS],
    key_hash: Field,
    proof: UltraHonkProof,
    num: u64,
) -> pub u64 {
    // verify sum so far was computed correctly
    verify_ultrahonk_proof(
        verification_key,
        proof,
        public_inputs,
        key_hash,
    );

    // Take output of previous proof and add another number to it.
    public_inputs[2] as u64 + num
}
