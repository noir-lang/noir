use std::path::{Path, PathBuf};

use crate::BackendError;

/// WriteCommand will call the barretenberg binary
/// to write a verification key to a file
pub(crate) struct WriteVkCommand {
    pub(crate) crs_path: PathBuf,
    pub(crate) is_recursive: bool,
    pub(crate) bytecode_path: PathBuf,
    pub(crate) vk_path_output: PathBuf,
}

impl WriteVkCommand {
    pub(crate) fn run(self, binary_path: &Path) -> Result<(), BackendError> {
        let mut command = std::process::Command::new(binary_path);

        command
            .arg("write_vk")
            .arg("-c")
            .arg(self.crs_path)
            .arg("-b")
            .arg(self.bytecode_path)
            .arg("-o")
            .arg(self.vk_path_output);

        if self.is_recursive {
            command.arg("-r");
        }

        let output = command.output().expect("Failed to execute command");
        if output.status.success() {
            Ok(())
        } else {
            Err(BackendError(String::from_utf8(output.stderr).unwrap()))
        }
    }
}

#[test]
fn write_vk_command() {
    use tempfile::tempdir;

    let backend = crate::get_mock_backend();

    let bytecode_path = PathBuf::from("./src/1_mul.bytecode");

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let vk_path_output = temp_directory.path().join("vk");

    let crs_path = backend.backend_directory();

    let write_vk_command =
        WriteVkCommand { bytecode_path, crs_path, is_recursive: false, vk_path_output };

    let vk_written = write_vk_command.run(&backend.binary_path());
    assert!(vk_written.is_ok());
    drop(temp_directory);
}
