#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]

use std::{collections::HashSet, path::PathBuf};

mod cli;
mod download;
mod proof_system;
mod smart_contract;

use acvm::acir::circuit::Opcode;
use bb_abstraction_leaks::ACVM_BACKEND_BARRETENBERG;
use bb_abstraction_leaks::BB_VERSION;
use cli::VersionCommand;
pub use download::download_backend;

const BACKENDS_DIR: &str = ".nargo/backends";

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

    #[error("The backend responded with a malformed UTF8 byte vector: {0:?}")]
    InvalidUTF8Vector(Vec<u8>),

    #[error(
        "The backend responded with a unexpected number of bytes. Expected: {0} but got {} ({1:?})", .1.len()
    )]
    UnexpectedNumberOfBytes(usize, Vec<u8>),

    #[error("The backend encountered an error: {0:?}")]
    CommandFailed(String),
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

    pub fn name(&self) -> &str {
        &self.name
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
                let bb_url = std::env::var("BB_BINARY_URL")
                    .unwrap_or_else(|_| bb_abstraction_leaks::BB_DOWNLOAD_URL.to_owned());
                download_backend(&bb_url, binary_path)?;
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

    fn assert_correct_version(&self) -> Result<&PathBuf, BackendError> {
        let binary_path = self.binary_path();
        if binary_path.to_string_lossy().contains(ACVM_BACKEND_BARRETENBERG) {
            match VersionCommand.run(binary_path) {
                // If version matches then do nothing.
                Ok(version_string) if version_string == BB_VERSION => (),

                // If version doesn't match then download the correct version.
                Ok(version_string) => {
                    log::warn!("`{ACVM_BACKEND_BARRETENBERG}` version `{version_string}` is different from expected `{BB_VERSION}`. Downloading expected version...");
                    let bb_url = std::env::var("BB_BINARY_URL")
                        .unwrap_or_else(|_| bb_abstraction_leaks::BB_DOWNLOAD_URL.to_owned());
                    download_backend(&bb_url, binary_path)?;
                }

                // If `bb` fails to report its version, then attempt to fix it by re-downloading the binary.
                Err(_) => {
                    log::warn!("Could not determine version of `{ACVM_BACKEND_BARRETENBERG}`. Downloading expected version...");
                    let bb_url = std::env::var("BB_BINARY_URL")
                        .unwrap_or_else(|_| bb_abstraction_leaks::BB_DOWNLOAD_URL.to_owned());
                    download_backend(&bb_url, binary_path)?;
                }
            }
        }
        Ok(binary_path)
    }
}

pub struct BackendOpcodeSupport {
    opcodes: HashSet<String>,
    black_box_functions: HashSet<String>,
}

impl BackendOpcodeSupport {
    pub fn is_opcode_supported(&self, opcode: &Opcode) -> bool {
        match opcode {
            Opcode::Arithmetic(_) => self.opcodes.contains("arithmetic"),
            Opcode::Directive(_) => self.opcodes.contains("directive"),
            Opcode::Brillig(_) => self.opcodes.contains("brillig"),
            Opcode::MemoryInit { .. } => self.opcodes.contains("memory_init"),
            Opcode::MemoryOp { .. } => self.opcodes.contains("memory_op"),
            Opcode::BlackBoxFuncCall(func) => {
                self.black_box_functions.contains(func.get_black_box_func().name())
            }
        }
    }

    pub fn all() -> BackendOpcodeSupport {
        BackendOpcodeSupport {
            opcodes: HashSet::from([
                "arithmetic".to_string(),
                "directive".to_string(),
                "brillig".to_string(),
                "memory_init".to_string(),
                "memory_op".to_string(),
            ]),
            black_box_functions: HashSet::from([
                "sha256".to_string(),
                "schnorr_verify".to_string(),
                "blake2s".to_string(),
                "pedersen".to_string(),
                "pedersen_hash".to_string(),
                "hash_to_field_128_security".to_string(),
                "ecdsa_secp256k1".to_string(),
                "fixed_base_scalar_mul".to_string(),
                "and".to_string(),
                "xor".to_string(),
                "range".to_string(),
                "keccak256".to_string(),
                "recursive_aggregation".to_string(),
                "ecdsa_secp256r1".to_string(),
            ]),
        }
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
