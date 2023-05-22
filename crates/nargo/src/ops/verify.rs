use acvm::acir::circuit::Circuit;
use acvm::ProofSystemCompiler;
use noirc_abi::WitnessMap;

pub fn verify_proof<B: ProofSystemCompiler>(
    backend: &B,
    circuit: &Circuit,
    proof: &[u8],
    public_inputs: WitnessMap,
    verification_key: &[u8],
) -> Result<bool, B::Error> {
    backend.verify_with_vk(proof, public_inputs, circuit, verification_key)
}
