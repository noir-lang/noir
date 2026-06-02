//! Proof JSON format produced by the usb-auth web/CLI frontend.

use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofJson {
    /// Whether the proof was verified at generation time by the web backend.
    pub verified: bool,
    /// The nullifier (return value of the circuit).
    pub nullifier: String,
    /// Named public inputs from the Noir circuit.
    pub public_inputs: HashMap<String, String>,
    /// Raw proof bytes (UltraHonk serialization).
    pub proof: Vec<u8>,
    /// Proof public inputs as hex strings.
    pub proof_public_inputs: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ProofError {
    #[error("Cannot read proof file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid proof JSON: {0}")]
    Json(#[from] serde_json::Error),
}

impl ProofJson {
    pub fn from_file(path: &Path) -> Result<Self, ProofError> {
        let text = std::fs::read_to_string(path)?;
        let proof: Self = serde_json::from_str(&text)?;
        Ok(proof)
    }

    /// Returns the `usb_serial` public input as a string, if present.
    pub fn usb_serial(&self) -> Option<&str> {
        self.public_inputs.get("usb_serial").map(String::as_str)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_proof_json() -> ProofJson {
        let json = r#"{
            "verified": true,
            "nullifier": "1234",
            "publicInputs": {
                "usb_serial": "5678",
                "commitment": "9999",
                "challenge": "11",
                "user_id_hash": "22",
                "expected_nullifier": "1234"
            },
            "proof": [1, 2, 3],
            "proofPublicInputs": ["0x01", "0x02"]
        }"#;
        serde_json::from_str(json).expect("fixture is valid")
    }

    #[test]
    fn parses_usb_serial_from_public_inputs() {
        let proof = sample_proof_json();
        assert_eq!(proof.usb_serial(), Some("5678"));
    }

    #[test]
    fn parses_verified_flag() {
        let proof = sample_proof_json();
        assert!(proof.verified);
    }

    #[test]
    fn parses_nullifier() {
        let proof = sample_proof_json();
        assert_eq!(proof.nullifier, "1234");
    }

    #[test]
    fn rejects_invalid_json() {
        let err = serde_json::from_str::<ProofJson>("not json").unwrap_err();
        assert!(err.is_syntax());
    }
}
