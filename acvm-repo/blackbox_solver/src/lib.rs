#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

//! This crate provides the implementation of BlackBox functions of ACIR and Brillig.
//! For functions that are backend-dependent, it provides a Trait [BlackBoxFunctionSolver] that must be implemented by the backend.
//! For functions that have a reference implementation, such as [keccakf1600], this crate exports the reference implementation directly.

use acir::BlackBoxFunc;
use thiserror::Error;

mod aes128;
mod bigint;
mod curve_specific_solver;
mod ecdsa;
mod hash;
mod logic;

pub use aes128::aes128_encrypt;
pub use bigint::{BigIntSolver, BigIntSolverWithId};
pub use curve_specific_solver::{BlackBoxFunctionSolver, StubbedBlackBoxSolver};
pub use ecdsa::{ecdsa_secp256k1_verify, ecdsa_secp256r1_verify};
pub use hash::{blake2s, blake3, keccakf1600, sha256_compression};
pub use logic::{bit_and, bit_xor};

#[derive(Clone, PartialEq, Eq, Debug, Error)]
pub enum BlackBoxResolutionError {
    #[error("failed to solve blackbox function: {0}, reason: {1}")]
    Failed(BlackBoxFunc, String),
}
