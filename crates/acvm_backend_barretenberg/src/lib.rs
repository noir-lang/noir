#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]

mod bb;
mod proof_system;
mod smart_contract;

#[derive(Debug, Default)]
pub struct Backend {}

impl Backend {
    pub fn new() -> Backend {
        Backend {}
    }
}

#[derive(Debug, thiserror::Error)]
pub struct BackendError(String);

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
