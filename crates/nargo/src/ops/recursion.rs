use acvm::{ProofSystemCompiler, FieldElement};
use noirc_abi::WitnessMap;

use crate::NargoError;

pub fn proof_as_fields(
    backend: &impl ProofSystemCompiler,
    proof: &[u8],
    public_inputs: WitnessMap,
) -> Result<Vec<FieldElement>, NargoError> {
    let proof_as_fields = backend.proof_as_fields(proof, public_inputs);
    
    Ok(proof_as_fields)
}

pub fn vk_as_fields(
    backend: &impl ProofSystemCompiler,
    verification_key: &[u8],
) -> Result<(Vec<FieldElement>, FieldElement), NargoError> {
    let (vk_as_fields, vk_hash) = backend.vk_as_fields(verification_key);
    
    Ok((vk_as_fields, vk_hash))
}