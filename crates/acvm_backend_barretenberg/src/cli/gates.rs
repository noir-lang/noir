use std::path::{Path, PathBuf};

use crate::BackendError;

/// GatesCommand will call the barretenberg binary
/// to return the number of gates needed to create a proof
/// for the given bytecode.
pub(crate) struct GatesCommand {
    pub(crate) crs_path: PathBuf,
    pub(crate) bytecode_path: PathBuf,
}

impl GatesCommand {
    pub(crate) fn run(self, binary_path: &Path) -> Result<u32, BackendError> {
        let output = std::process::Command::new(binary_path)
            .arg("gates")
            .arg("-c")
            .arg(self.crs_path)
            .arg("-b")
            .arg(self.bytecode_path)
            .output()?;

        if !output.status.success() {
            return Err(BackendError::CommandFailed(output.stderr));
        }
        // Note: barretenberg includes the newline, so that subsequent prints to stdout
        // are not on the same line as the gates output.

        let gates_bytes: [u8; 8] =
            output.stdout.try_into().map_err(BackendError::MalformedResponse)?;

        // Convert bytes to u64 in little-endian format
        let value = u64::from_le_bytes(gates_bytes);

        Ok(value as u32)
    }
}

#[test]
fn gate_command() -> Result<(), BackendError> {
    use tempfile::tempdir;

    let backend = crate::get_mock_backend()?;

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();
    let bytecode_path = temp_directory_path.join("acir.gz");
    let crs_path = backend.backend_directory();

    std::fs::File::create(&bytecode_path).expect("file should be created");

    let gate_command = GatesCommand { crs_path, bytecode_path };

    let output = gate_command.run(&backend.binary_path())?;
    // Mock backend always returns zero gates.
    assert_eq!(output, 0);

    Ok(())
}
