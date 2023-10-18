use std::path::Path;

use crate::BackendError;

use super::string_from_stderr;

/// VersionCommand will call the backend binary
/// to query installed version.
pub(crate) struct VersionCommand;

impl VersionCommand {
    pub(crate) fn run(self, binary_path: &Path) -> Result<String, BackendError> {
        let mut command = std::process::Command::new(binary_path);

        command.arg("--version");

        let output = command.output()?;
        if output.status.success() {
            match String::from_utf8(output.stdout) {
                Ok(result) => Ok(result),
                Err(_) => Err(BackendError::CommandFailed(
                    "Unexpected output from --version check.".to_owned(),
                )),
            }
        } else {
            Err(BackendError::CommandFailed(string_from_stderr(&output.stderr)))
        }
    }
}
