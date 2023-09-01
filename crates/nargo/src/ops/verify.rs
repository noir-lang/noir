use acvm::acir::{circuit::Circuit, native_types::WitnessMap};
use acvm_backend_barretenberg::{Backend, BackendError};

pub fn verify_proof(
    backend: &Backend,
    circuit: &Circuit,
    proof: &[u8],
    public_inputs: WitnessMap,
) -> Result<bool, BackendError> {
    // TODO(#1569): update from not just accepting `false` once we get nargo to interop with dynamic backend
    backend.verify(proof, public_inputs, circuit, false)
}
