use acvm::acir::{circuit::Circuit, native_types::WitnessMap};
use acvm::ProofSystemCompiler;

pub fn verify_proof<B: ProofSystemCompiler>(
    backend: &B,
    circuit: &Circuit,
    proof: &[u8],
    public_inputs: WitnessMap,
) -> Result<bool, B::Error> {
    // TODO(#1569): update from not just accepting `false` once we get nargo to interop with dynamic backend
    // Nargo no longer handles logic related to proving/verifying with keys or the CRS.
    let common_reference_string = Vec::new();
    let verification_key = Vec::new();
    backend.verify_with_vk(
        &common_reference_string,
        proof,
        public_inputs,
        circuit,
        &verification_key,
        false,
    )
}
