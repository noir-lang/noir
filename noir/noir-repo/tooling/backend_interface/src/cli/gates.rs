use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::BackendError;

use super::string_from_stderr;

/// GatesCommand will call the barretenberg binary
/// to return the number of gates needed to create a proof
/// for the given bytecode.
pub(crate) struct GatesCommand {
    pub(crate) crs_path: PathBuf,
    pub(crate) artifact_path: PathBuf,
}

#[derive(Deserialize)]
struct GatesResponse {
    functions: Vec<CircuitReport>,
}

#[derive(Deserialize)]
pub struct CircuitReport {
    pub acir_opcodes: u32,
    pub circuit_size: u32,
}

impl GatesCommand {
    pub(crate) fn run(self, binary_path: &Path) -> Result<Vec<CircuitReport>, BackendError> {
        let output = std::process::Command::new(binary_path)
            .arg("gates")
            .arg("-c")
            .arg(self.crs_path)
            .arg("-b")
            .arg(self.artifact_path)
            .output()?;

        if !output.status.success() {
            return Err(BackendError::CommandFailed(string_from_stderr(&output.stderr)));
        }

        let gates_info: GatesResponse =
            serde_json::from_slice(&output.stdout).expect("Backend should return valid json");

        Ok(gates_info.functions)
    }
}

#[test]
fn gate_command() -> Result<(), BackendError> {
    use tempfile::tempdir;

    let backend = crate::get_mock_backend()?;

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();
    let artifact_path = temp_directory_path.join("program.json");
    let crs_path = backend.backend_directory();

    std::fs::File::create(&artifact_path).expect("file should be created");

    let gate_command = GatesCommand { crs_path, artifact_path };

    let output = gate_command.run(backend.binary_path())?;
    // Mock backend always returns zero gates.
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].acir_opcodes, 123);
    assert_eq!(output[0].circuit_size, 125);

    Ok(())
}
