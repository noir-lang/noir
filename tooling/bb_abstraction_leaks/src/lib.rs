#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]

pub const ACVM_BACKEND_BARRETENBERG: &str = "acvm-backend-barretenberg";
pub const BB_DOWNLOAD_URL: &str = env!("BB_BINARY_URL");

/// Embed the Solidity verifier file
const ULTRA_VERIFIER_CONTRACT: &str = include_str!("contract.sol");

pub fn complete_barretenberg_verifier_contract(contract: String) -> String {
    format!("{contract}{ULTRA_VERIFIER_CONTRACT}")
}
