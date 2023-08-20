use acvm::{acir::circuit::Circuit, SmartContract};

pub fn codegen_verifier<B: SmartContract>(
    backend: &B,
    circuit: &Circuit,
) -> Result<String, B::Error> {
    // Nargo no longer handles logic related to proving/verifying with keys or the CRS.
    let common_reference_string = Vec::new();
    let verification_key = Vec::new();
    backend.eth_contract_from_vk(&common_reference_string, circuit, &verification_key)
}
