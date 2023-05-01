use acvm::acir::{circuit::Circuit, native_types::WitnessMap};
use acvm::ProofSystemCompiler;

use crate::NargoError;

pub fn prove_execution(
    backend: &impl ProofSystemCompiler,
    circuit: &Circuit,
    solved_witness: WitnessMap,
    proving_key: &[u8],
) -> Result<Vec<u8>, NargoError> {
    let proof = backend.prove_with_pk(circuit, solved_witness, proving_key);

    Ok(proof)
}
