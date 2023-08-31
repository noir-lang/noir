#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]

mod bb;
mod proof_system;
mod smart_contract;

#[derive(Debug, Default)]
pub struct Barretenberg;

impl Barretenberg {
    pub fn new() -> Barretenberg {
        Barretenberg
    }
}

impl acvm::Backend for Barretenberg {}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct BackendError(#[from] Error);

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, thiserror::Error)]
enum Error {}
