use acvm::{acir::circuit::Circuit, SmartContract};

pub fn codegen_verifier<B: SmartContract>(
    backend: &B,
    common_reference_string: &[u8],
    circuit: &Circuit,
    verification_key: &[u8],
) -> Result<String, B::Error> {
    backend.eth_contract_from_vk(common_reference_string, circuit, verification_key)
}
