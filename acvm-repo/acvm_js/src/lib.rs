#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

// See Cargo.toml for explanation.
use getrandom as _;

mod black_box_solvers;
mod build_info;
mod compression;
mod execute;
mod foreign_call;
mod js_execution_error;
mod js_witness_map;
mod js_witness_stack;
mod logging;
mod public_witness;

pub use black_box_solvers::{
    and, blake2s256, ecdsa_secp256k1_verify, ecdsa_secp256r1_verify, sha256_compression, xor,
};
pub use build_info::build_info;
pub use compression::{
    compress_witness, compress_witness_stack, decompress_witness, decompress_witness_stack,
};
pub use execute::{execute_circuit, execute_circuit_with_return_witness, execute_program};
pub use js_execution_error::JsExecutionError;
pub use js_witness_map::JsSolvedAndReturnWitness;
pub use js_witness_map::JsWitnessMap;
pub use js_witness_stack::JsWitnessStack;
pub use logging::init_log_level;
pub use public_witness::{get_public_parameters_witness, get_public_witness, get_return_witness};
