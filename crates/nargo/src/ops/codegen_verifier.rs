use acvm::{acir::circuit::Circuit, SmartContract};

pub fn codegen_verifier<B: SmartContract>(
    backend: &B,
    circuit: &Circuit,
) -> Result<String, B::Error> {
    backend.eth_contract(circuit)
}
