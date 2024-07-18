// docs:start:oracles-module
/// Oracles module
// docs:end:oracles-module

mod arguments;
mod call_private_function;
mod encryption;
mod execution;
mod get_contract_instance;
mod get_l1_to_l2_membership_witness;
mod get_nullifier_membership_witness;
mod get_public_data_witness;
mod get_membership_witness;
mod keys;
mod key_validation_request;
mod get_sibling_path;
mod unsafe_rand;
mod enqueue_public_function_call;
mod header;
mod public_call;
mod notes;
mod storage;
mod logs;
mod logs_traits;
mod returns;

// debug_log oracle is used by both noir-protocol-circuits and this crate and for this reason we just re-export it
// here from protocol circuits.
use dep::protocol_types::debug_log;
