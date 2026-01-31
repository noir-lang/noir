//! Cryptographic utilities for the private token client

use sha2::{Sha256, Digest};
use rand::RngCore;

/// Generate a random 32-byte secret
pub fn generate_secret() -> [u8; 32] {
    let mut secret = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut secret);
    secret
}

/// Compute a simple hash (placeholder for Pedersen hash)
/// In production, use the actual Pedersen hash from Noir/Barretenberg
pub fn pedersen_hash(inputs: &[&[u8; 32]]) -> [u8; 32] {
    // This is a placeholder using SHA256
    // TODO: Replace with actual Pedersen hash from bn254_blackbox_solver
    let mut hasher = Sha256::new();
    for input in inputs {
        hasher.update(input);
    }
    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

/// Derive address from secret key
pub fn derive_address(secret: &[u8; 32]) -> [u8; 32] {
    pedersen_hash(&[secret])
}

/// Compute commitment: Hash(address, balance, nonce)
pub fn compute_commitment(address: &[u8; 32], balance: u128, nonce: u64) -> [u8; 32] {
    let balance_bytes = u128_to_bytes32(balance);
    let nonce_bytes = u64_to_bytes32(nonce);
    pedersen_hash(&[address, &balance_bytes, &nonce_bytes])
}

/// Compute nullifier: Hash(secret, nonce)
pub fn compute_nullifier(secret: &[u8; 32], nonce: u64) -> [u8; 32] {
    let nonce_bytes = u64_to_bytes32(nonce);
    pedersen_hash(&[secret, &nonce_bytes])
}

/// Convert u128 to 32-byte array (big-endian, left-padded)
pub fn u128_to_bytes32(value: u128) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[16..].copy_from_slice(&value.to_be_bytes());
    bytes
}

/// Convert u64 to 32-byte array (big-endian, left-padded)
pub fn u64_to_bytes32(value: u64) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[24..].copy_from_slice(&value.to_be_bytes());
    bytes
}

/// Convert bytes32 to hex string
pub fn bytes32_to_hex(bytes: &[u8; 32]) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// Convert hex string to bytes32
pub fn hex_to_bytes32(hex_str: &str) -> Result<[u8; 32], hex::FromHexError> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let bytes = hex::decode(hex_str)?;
    let mut result = [0u8; 32];
    if bytes.len() == 32 {
        result.copy_from_slice(&bytes);
    } else if bytes.len() < 32 {
        result[32 - bytes.len()..].copy_from_slice(&bytes);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secret() {
        let secret1 = generate_secret();
        let secret2 = generate_secret();
        assert_ne!(secret1, secret2);
    }

    #[test]
    fn test_derive_address() {
        let secret = generate_secret();
        let address1 = derive_address(&secret);
        let address2 = derive_address(&secret);
        assert_eq!(address1, address2);
    }

    #[test]
    fn test_commitment_deterministic() {
        let address = generate_secret();
        let balance = 100u128;
        let nonce = 1u64;
        
        let commitment1 = compute_commitment(&address, balance, nonce);
        let commitment2 = compute_commitment(&address, balance, nonce);
        assert_eq!(commitment1, commitment2);
    }

    #[test]
    fn test_nullifier_deterministic() {
        let secret = generate_secret();
        let nonce = 1u64;
        
        let nullifier1 = compute_nullifier(&secret, nonce);
        let nullifier2 = compute_nullifier(&secret, nonce);
        assert_eq!(nullifier1, nullifier2);
    }

    #[test]
    fn test_hex_conversion() {
        let bytes = generate_secret();
        let hex_str = bytes32_to_hex(&bytes);
        let recovered = hex_to_bytes32(&hex_str).unwrap();
        assert_eq!(bytes, recovered);
    }
}
