use std::path::{Path, PathBuf};

use super::string_from_stderr;
use crate::BackendError;

/// WriteCommand will call the barretenberg binary
/// to write a verification key to a file
pub(crate) struct WriteVkCommand {
    pub(crate) crs_path: PathBuf,
    pub(crate) artifact_path: PathBuf,
    pub(crate) vk_path_output: PathBuf,
}

impl WriteVkCommand {
    #[tracing::instrument(level = "trace", name = "vk_generation", skip_all)]
    pub(crate) fn run(self, binary_path: &Path) -> Result<(), BackendError> {
        let mut command = std::process::Command::new(binary_path);

        command
            .arg("write_vk")
            .arg("-c")
            .arg(self.crs_path)
            .arg("-b")
            .arg(self.artifact_path)
            .arg("-o")
            .arg(self.vk_path_output);

        let output = command.output()?;
        if output.status.success() {
            Ok(())
        } else {
            Err(BackendError::CommandFailed(string_from_stderr(&output.stderr)))
        }
    }
}

#[test]
fn write_vk_command() -> Result<(), BackendError> {
    use tempfile::tempdir;

    let backend = crate::get_mock_backend()?;

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();
    let artifact_path = temp_directory_path.join("program.json");
    let vk_path_output = temp_directory.path().join("vk");

    let crs_path = backend.backend_directory();

    std::fs::File::create(&artifact_path).expect("file should be created");

    let write_vk_command = WriteVkCommand { artifact_path, crs_path, vk_path_output };

    write_vk_command.run(backend.binary_path())?;
    drop(temp_directory);

    Ok(())
}
