use bb_proof_verification::{verify_ultrahonk_proof, UltraHonkProof, UltraHonkVerificationKey};


fn main(
    verification_key: UltraHonkVerificationKey,
    proof: UltraHonkProof,
    public_inputs: [Field; 2]
) {
    verify_ultrahonk_proof(
        verification_key,
        proof,
        public_inputs,
        0x0,
    );
}
