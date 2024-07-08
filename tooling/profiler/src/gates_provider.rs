use std::path::{Path, PathBuf};
use std::process::Command;

use color_eyre::eyre::{self};
use serde::{Deserialize, Serialize};

pub(crate) trait GatesProvider {
    fn get_gates(&self, artifact_path: &Path) -> eyre::Result<BackendGatesResponse>;
}

pub(crate) struct BackendGatesProvider {
    pub(crate) backend_path: PathBuf,
}

impl GatesProvider for BackendGatesProvider {
    fn get_gates(&self, artifact_path: &Path) -> eyre::Result<BackendGatesResponse> {
        let backend_gates_response =
            Command::new(&self.backend_path).arg("gates").arg("-b").arg(artifact_path).output()?;

        // Parse the backend gates command stdout as json
        let backend_gates_response: BackendGatesResponse =
            serde_json::from_slice(&backend_gates_response.stdout)?;
        Ok(backend_gates_response)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BackendGatesReport {
    pub(crate) acir_opcodes: usize,
    pub(crate) circuit_size: usize,
    pub(crate) gates_per_opcode: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BackendGatesResponse {
    pub(crate) functions: Vec<BackendGatesReport>,
}
