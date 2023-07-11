use acvm::SmartContract;

pub fn codegen_verifier<B: SmartContract>(
    backend: &B,
    common_reference_string: &[u8],
    verification_key: &[u8],
) -> Result<String, B::Error> {
    backend.eth_contract_from_vk(common_reference_string, verification_key)
}
