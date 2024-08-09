use std::path::{Path, PathBuf};
use std::process::Command;

use color_eyre::eyre::{self};
use serde::{Deserialize, Serialize};

pub(crate) trait GatesProvider {
    fn get_gates(&self, artifact_path: &Path) -> eyre::Result<BackendGatesResponse>;
}

pub(crate) struct BackendGatesProvider {
    pub(crate) backend_path: PathBuf,
    pub(crate) extra_args: Vec<String>,
}

impl GatesProvider for BackendGatesProvider {
    fn get_gates(&self, artifact_path: &Path) -> eyre::Result<BackendGatesResponse> {
        let mut backend_gates_cmd = Command::new(&self.backend_path);

        backend_gates_cmd.arg("gates").arg("-b").arg(artifact_path);

        for arg in &self.extra_args {
            backend_gates_cmd.arg(arg);
        }

        let backend_gates_response = backend_gates_cmd.output()?;

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
