use acvm::acir::circuit::Circuit;
use acvm::ProofSystemCompiler;
use noirc_abi::WitnessMap;

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
