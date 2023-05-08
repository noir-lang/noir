use acvm::SmartContract;

pub fn codegen_verifier<Backend: SmartContract>(
    backend: &Backend,
    verification_key: &[u8],
) -> Result<String, Backend::Error> {
    backend.eth_contract_from_vk(verification_key)
}
