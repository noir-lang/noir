#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]

use std::path::PathBuf;

// `acvm-backend-barretenberg` can either interact with the Barretenberg backend through a static library
// or through an embedded wasm binary. It does not make sense to include both of these backends at the same time.
// We then throw a compilation error if both flags are set.
#[cfg(all(feature = "native", feature = "wasm"))]
compile_error!("feature \"native\" and feature \"wasm\" cannot be enabled at the same time");

#[cfg(all(feature = "native", target_arch = "wasm32"))]
compile_error!("feature \"native\" cannot be enabled for a \"wasm32\" target");

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
compile_error!("feature \"wasm\" cannot be enabled for a \"wasm32\" target");

mod bb;
mod cli;
mod proof_system;
mod smart_contract;

/// The number of bytes necessary to store a `FieldElement`.
const FIELD_BYTES: usize = 32;

fn get_bb() -> Backend {
    Backend::new("acvm-backend-barretenberg".to_owned())
}

fn assert_binary_exists() -> PathBuf {
    let bb = get_bb();
    let binary_path = bb.binary_path();

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
        const BACKENDS_DIR: &str = ".nargo/backends";

        let home_directory = dirs::home_dir().unwrap();

        home_directory.join(BACKENDS_DIR).join(&self.name)
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
