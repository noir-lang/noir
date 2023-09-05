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
test_binary::build_test_binary_once!(mock_backend, "test-binaries");

#[cfg(test)]
fn get_mock_backend() -> Backend {
    std::env::set_var("NARGO_BACKEND_PATH", path_to_mock_backend());

    let bb = Backend::new("acvm-backend-barretenberg".to_string());
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

#[derive(Debug)]
pub struct Backend {
    name: String,
}

impl Backend {
    pub fn new(name: String) -> Backend {
        Backend { name }
    }

    fn backend_directory(&self) -> PathBuf {
        backends_directory().join(&self.name)
    }

    fn binary_path(&self) -> PathBuf {
        if let Some(binary_path) = std::env::var_os("NARGO_BACKEND_PATH") {
            PathBuf::from(binary_path)
        } else {
            const BINARY_NAME: &str = "backend_binary";

            self.backend_directory().join(BINARY_NAME)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub struct BackendError(String);

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
