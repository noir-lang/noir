//! Error types for the private token client

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: u128, need: u128 },

    #[error("Commitment not found: {0}")]
    CommitmentNotFound(String),

    #[error("State error: {0}")]
    StateError(String),

    #[error("Proof generation failed: {0}")]
    ProofError(String),

    #[error("Contract error: {0}")]
    ContractError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Hex decode error: {0}")]
    HexError(#[from] hex::FromHexError),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, ClientError>;
