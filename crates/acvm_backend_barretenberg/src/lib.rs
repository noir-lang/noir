#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]

use std::path::PathBuf;

mod bb;
mod cli;
mod proof_system;
mod smart_contract;

fn assert_binary_exists() -> PathBuf {
    let binary_path = bb::get_binary_path();
    if !binary_path.is_file() {
        bb::download_bb_binary(&binary_path)
    }
    binary_path
}

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
