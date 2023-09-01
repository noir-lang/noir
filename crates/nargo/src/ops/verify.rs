use acvm::acir::{circuit::Circuit, native_types::WitnessMap};
use acvm::ProofSystemCompiler;

pub fn verify_proof<B: ProofSystemCompiler>(
    backend: &B,
    circuit: &Circuit,
    proof: &[u8],
    public_inputs: WitnessMap,
) -> Result<bool, B::Error> {
    // TODO(#1569): update from not just accepting `false` once we get nargo to interop with dynamic backend
    backend.verify(proof, public_inputs, circuit, false)
}
