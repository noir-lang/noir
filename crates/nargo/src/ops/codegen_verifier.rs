use acvm::acir::circuit::Circuit;
use acvm_backend_barretenberg::{Backend, BackendError};

pub fn codegen_verifier(backend: &Backend, circuit: &Circuit) -> Result<String, BackendError> {
    backend.eth_contract(circuit)
}
