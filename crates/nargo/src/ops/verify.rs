use acvm::acir::{circuit::Circuit, native_types::WitnessMap};
use acvm::ProofSystemCompiler;

pub fn verify_proof<B: ProofSystemCompiler>(
    backend: &B,
    common_reference_string: &[u8],
    circuit: &Circuit,
    proof: &[u8],
    public_inputs: WitnessMap,
) -> Result<bool, B::Error> {
    // TODO(#1569): update from not just accepting `false` once we get nargo to interop with dynamic backend
    backend.verify_with_vk(
        common_reference_string,
        proof,
        public_inputs,
        circuit,
        &Vec::new(),
        false,
    )
}
