use acvm::acir::{circuit::Circuit, native_types::WitnessMap};
use acvm::ProofSystemCompiler;

pub fn prove_execution<B: ProofSystemCompiler>(
    backend: &B,
    circuit: &Circuit,
    solved_witness: WitnessMap,
) -> Result<Vec<u8>, B::Error> {
    // TODO(#1569): update from not just accepting `false` once we get nargo to interop with dynamic backend
    backend.prove(circuit, solved_witness, false)
}
