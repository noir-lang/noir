use bb_proof_verification::{UltraHonkVerificationKey, UltraHonkZKProof, verify_ultrahonkzk_proof};

fn main(
    verification_key: UltraHonkVerificationKey,
    proof: UltraHonkZKProof,
    public_inputs: [Field; 2],
    key_hash: Field,
) {
    verify_ultrahonkzk_proof(verification_key, proof, public_inputs, key_hash);
}
