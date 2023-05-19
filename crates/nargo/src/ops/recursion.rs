use acvm::{acir::native_types::WitnessMap, FieldElement, ProofSystemCompiler};

pub fn proof_as_fields<B: ProofSystemCompiler>(
    backend: &B,
    proof: &[u8],
    public_inputs: WitnessMap,
) -> Result<Vec<FieldElement>, B::Error> {
    let proof_as_fields = backend.proof_as_fields(proof, public_inputs)?;

    Ok(proof_as_fields)
}

pub fn vk_as_fields<B: ProofSystemCompiler>(
    backend: &B,
    common_reference_string: &[u8],
    verification_key: &[u8],
) -> Result<(Vec<FieldElement>, FieldElement), B::Error> {
    let (vk_as_fields, vk_hash) =
        backend.vk_as_fields(common_reference_string, verification_key)?;

    Ok((vk_as_fields, vk_hash))
}
