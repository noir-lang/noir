mod prover;
mod verifier;

pub use prover::process_abi_with_input;
pub use verifier::process_abi_with_verifier_input;

pub const MAIN_RETURN_NAME: &str = "return";
