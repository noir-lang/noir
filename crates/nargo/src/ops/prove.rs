use acvm::acir::{circuit::Circuit, native_types::WitnessMap};
use acvm_backend_barretenberg::{Backend, BackendError};

pub fn prove_execution(
    backend: &Backend,
    circuit: &Circuit,
    solved_witness: WitnessMap,
) -> Result<Vec<u8>, BackendError> {
    // TODO(#1569): update from not just accepting `false` once we get nargo to interop with dynamic backend
    backend.prove(circuit, solved_witness, false)
}
