#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]

use acvm::FieldElement;

pub const ACVM_BACKEND_BARRETENBERG: &str = "acvm-backend-barretenberg";
pub const BB_DOWNLOAD_URL: &str = env!("BB_BINARY_URL");

/// Embed the Solidity verifier file
const ULTRA_VERIFIER_CONTRACT: &str = include_str!("contract.sol");

pub fn complete_barretenberg_verifier_contract(contract: String) -> String {
    format!("{contract}{ULTRA_VERIFIER_CONTRACT}")
}

/// Removes the public inputs which are prepended to a proof by Barretenberg.
pub fn remove_public_inputs(num_pub_inputs: usize, proof: &[u8]) -> Vec<u8> {
    // Barretenberg prepends the public inputs onto the proof so we need to remove
    // the first `num_pub_inputs` field elements.
    let num_bytes_to_remove = num_pub_inputs * (FieldElement::max_num_bytes() as usize);
    proof[num_bytes_to_remove..].to_vec()
}

/// Prepends a set of public inputs to a proof.
pub fn prepend_public_inputs(proof: Vec<u8>, public_inputs: Vec<FieldElement>) -> Vec<u8> {
    if public_inputs.is_empty() {
        return proof;
    }

    let public_inputs_bytes =
        public_inputs.into_iter().flat_map(|assignment| assignment.to_be_bytes());

    public_inputs_bytes.chain(proof).collect()
}
