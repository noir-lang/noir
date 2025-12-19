global NUM_NON_ACCUMULATOR_PUBLIC_INPUTS: u32 = 4;

use bb_proof_verification::{verify_ultrahonkzk_proof, UltraHonkZKProof, UltraHonkVerificationKey};


fn main(
    verification_key: UltraHonkVerificationKey,
    public_inputs: pub [Field; NUM_NON_ACCUMULATOR_PUBLIC_INPUTS],
    key_hash: [Field; 1],
    proof: UltraHonkZKProof
) -> pub u64 {
    // verify sum was computed correctly
    verify_ultrahonkzk_proof(
        verification_key,
        proof,
        public_inputs,
        key_hash[0],
    );

    public_inputs[3] as u64
}
