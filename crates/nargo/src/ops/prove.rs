use acvm::acir::circuit::Circuit;
use acvm::ProofSystemCompiler;
use noirc_abi::WitnessMap;

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
