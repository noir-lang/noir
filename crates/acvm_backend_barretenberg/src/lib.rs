#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]

use std::path::PathBuf;

mod cli;
mod download;
mod proof_system;
mod smart_contract;

pub use download::download_backend;

const BACKENDS_DIR: &str = ".nargo/backends";
pub const ACVM_BACKEND_BARRETENBERG: &str = "acvm-backend-barretenberg";

pub fn backends_directory() -> PathBuf {
    let home_directory = dirs::home_dir().unwrap();
    home_directory.join(BACKENDS_DIR)
}

#[cfg(test)]
test_binary::build_test_binary_once!(mock_backend, "test-binaries");

#[cfg(test)]
fn get_mock_backend() -> Result<Backend, BackendError> {
    std::env::set_var("NARGO_BACKEND_PATH", path_to_mock_backend());

    let mock_backend = Backend::new("mock_backend".to_string());
    mock_backend.assert_binary_exists()?;

    Ok(mock_backend)
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Backend binary does not exist")]
    MissingBinary,

    #[error("The backend responded with malformed data: {0:?}")]
    MalformedResponse(Vec<u8>),

    #[error("The backend encountered an error")]
    CommandFailed(Vec<u8>),
}

#[derive(Debug)]
pub struct Backend {
    name: String,
    binary_path: PathBuf,
}

impl Backend {
    pub fn new(name: String) -> Backend {
        let binary_path = if let Some(binary_path) = std::env::var_os("NARGO_BACKEND_PATH") {
            PathBuf::from(binary_path)
        } else {
            const BINARY_NAME: &str = "backend_binary";

            backends_directory().join(&name).join(BINARY_NAME)
        };
        Backend { name, binary_path }
    }

    fn binary_path(&self) -> &PathBuf {
        &self.binary_path
    }

    fn assert_binary_exists(&self) -> Result<&PathBuf, BackendError> {
        let binary_path = self.binary_path();
        if binary_path.is_file() {
            Ok(binary_path)
        } else {
            if self.name == ACVM_BACKEND_BARRETENBERG {
                // If we're trying to use barretenberg, automatically go and install it.
                let bb_url =
                    std::env::var("BB_BINARY_URL").unwrap_or(env!("BB_BINARY_URL").to_string());
                download_backend(&bb_url, binary_path);
                return Ok(binary_path);
            }
            Err(BackendError::MissingBinary)
        }
    }

    fn backend_directory(&self) -> PathBuf {
        self.binary_path()
            .parent()
            .expect("backend binary should have a parent directory")
            .to_path_buf()
    }

    fn crs_directory(&self) -> PathBuf {
        self.backend_directory().join("crs")
    }
}

#[cfg(test)]
mod backend {
    use crate::{Backend, BackendError};

    #[test]
    fn raises_error_on_missing_binary() {
        let bad_backend = Backend::new("i_dont_exist".to_string());

        let binary_path = bad_backend.assert_binary_exists();

        assert!(matches!(binary_path, Err(BackendError::MissingBinary)));
    }
}
