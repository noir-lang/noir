use acvm::acir::{circuit::Circuit, native_types::WitnessMap};
use acvm::ProofSystemCompiler;

pub fn prove_execution<B: ProofSystemCompiler>(
    backend: &B,
    common_reference_string: &[u8],
    circuit: &Circuit,
    solved_witness: WitnessMap,
) -> Result<Vec<u8>, B::Error> {
    // TODO(#1569): update from not just accepting `false` once we get nargo to interop with dynamic backend
    backend.prove_with_pk(common_reference_string, circuit, solved_witness, &Vec::new(), false)
}
