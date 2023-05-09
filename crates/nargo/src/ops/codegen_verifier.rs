use acvm::SmartContract;

pub fn codegen_verifier<B: SmartContract>(
    backend: &B,
    verification_key: &[u8],
) -> Result<String, B::Error> {
    backend.eth_contract_from_vk(verification_key)
}
