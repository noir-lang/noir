use acvm::SmartContract;

use crate::NargoError;

pub fn codegen_verifier(
    backend: &impl SmartContract,
    verification_key: &[u8],
) -> Result<String, NargoError> {
    Ok(backend.eth_contract_from_vk(verification_key))
}
