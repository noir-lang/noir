#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]

use std::path::PathBuf;

mod bb;
mod cli;
mod proof_system;
mod smart_contract;

const BACKENDS_DIR: &str = ".nargo/backends";

pub fn backends_directory() -> PathBuf {
    let home_directory = dirs::home_dir().unwrap();
    home_directory.join(BACKENDS_DIR)
}

#[cfg(test)]
fn get_bb() -> Backend {
    let bb = Backend::new();
    crate::assert_binary_exists(&bb);
    bb
}

fn assert_binary_exists(backend: &Backend) -> PathBuf {
    let binary_path = backend.binary_path();

    if !binary_path.is_file() {
        bb::download_bb_binary(&binary_path)
    }
    binary_path
}

#[derive(Debug, Default)]
pub struct Backend {}

impl Backend {
    pub fn new() -> Backend {
        Backend::default()
    }

    fn backend_directory(&self) -> PathBuf {
        const BACKEND_NAME: &str = "acvm-backend-barretenberg";

        backends_directory().join(BACKEND_NAME)
    }

    fn binary_path(&self) -> PathBuf {
        const BINARY_NAME: &str = "backend_binary";

        self.backend_directory().join(BINARY_NAME)
    }
}

#[derive(Debug, thiserror::Error)]
pub struct BackendError(String);

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
