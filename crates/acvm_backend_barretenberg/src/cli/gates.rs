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
            .output()
            .expect("Failed to execute command");

        if !output.status.success() {
            return Err(BackendError(String::from_utf8(output.stderr).unwrap()));
        }
        // Note: barretenberg includes the newline, so that subsequent prints to stdout
        // are not on the same line as the gates output.

        // Ensure we got the expected number of bytes
        if output.stdout.len() != 8 {
            return Err(BackendError(format!(
                "Unexpected 8 bytes, received {}",
                output.stdout.len()
            )));
        }

        // Convert bytes to u64 in little-endian format
        let value = u64::from_le_bytes([
            output.stdout[0],
            output.stdout[1],
            output.stdout[2],
            output.stdout[3],
            output.stdout[4],
            output.stdout[5],
            output.stdout[6],
            output.stdout[7],
        ]);

        Ok(value as u32)
    }
}

#[test]
#[serial_test::serial]
fn gate_command() {
    let backend = crate::get_bb();
    let bytecode_path = PathBuf::from("./src/1_mul.bytecode");

    let crs_path = backend.backend_directory();

    let gate_command = GatesCommand { crs_path, bytecode_path };

    let output = gate_command.run(&backend.binary_path()).unwrap();
    assert_eq!(output, 2788);
}
