use acvm::acir::circuit::Circuit;
use acvm::ProofSystemCompiler;
use noirc_abi::WitnessMap;

pub fn prove_execution<Backend: ProofSystemCompiler>(
    backend: &Backend,
    circuit: &Circuit,
    solved_witness: WitnessMap,
    proving_key: &[u8],
) -> Result<Vec<u8>, Backend::Error> {
    backend.prove_with_pk(circuit, solved_witness, proving_key)
}
