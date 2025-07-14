global NUM_NON_ACCUMULATOR_PUBLIC_INPUTS: u32 = 4;

use bb_proof_verification::{verify_ultrahonk_proof, UltraHonkProof, UltraHonkVerificationKey};


fn main(
    verification_key: UltraHonkVerificationKey,
    public_inputs: pub [Field; NUM_NON_ACCUMULATOR_PUBLIC_INPUTS],
    key_hash: Field,
    proof: UltraHonkProof
) -> pub u64 {
    // verify sum was computed correctly
    verify_ultrahonk_proof(
        verification_key,
        proof,
        public_inputs,
        key_hash,
    );

    public_inputs[3] as u64
}
