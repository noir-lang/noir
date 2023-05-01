use acvm::acir::{circuit::Circuit, native_types::WitnessMap};
use acvm::ProofSystemCompiler;

use crate::NargoError;

pub fn verify_proof(
    backend: &impl ProofSystemCompiler,
    circuit: &Circuit,
    proof: &[u8],
    public_inputs: WitnessMap,
    verification_key: &[u8],
) -> Result<bool, NargoError> {
    let valid_proof = backend.verify_with_vk(proof, public_inputs, circuit, verification_key);

    Ok(valid_proof)
}
