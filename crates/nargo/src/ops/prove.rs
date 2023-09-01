use acvm::acir::{circuit::Circuit, native_types::WitnessMap};
use acvm::ProofSystemCompiler;

pub fn prove_execution<B: ProofSystemCompiler>(
    backend: &B,
    circuit: &Circuit,
    solved_witness: WitnessMap,
) -> Result<Vec<u8>, B::Error> {
    // TODO(#1569): update from not just accepting `false` once we get nargo to interop with dynamic backend
    // Nargo no longer handles logic related to proving/verifying with keys or the CRS.
    let common_reference_string = Vec::new();
    let proving_key = Vec::new();
    backend.prove_with_pk(&common_reference_string, circuit, solved_witness, &proving_key, false)
}
