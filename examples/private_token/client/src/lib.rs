//! Private Token Client
//! 
//! A Rust client for privacy-preserving token transactions using Noir ZK proofs.

pub mod state;
pub mod prover;
pub mod contract;
pub mod crypto;
pub mod error;

pub use state::StateManager;
pub use prover::ProofGenerator;
pub use contract::PrivateTokenContract;
pub use crypto::*;
pub use error::ClientError;
