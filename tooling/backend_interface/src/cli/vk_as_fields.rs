use std::path::{Path, PathBuf};

use acvm::FieldElement;

use crate::BackendError;

use super::string_from_stderr;

/// VkAsFieldsCommand will call the barretenberg binary
/// to split a verification key into a representation as [`FieldElement`]s.
///
/// The hash of the verification key will also be returned.
pub(crate) struct VkAsFieldsCommand {
    pub(crate) vk_path: PathBuf,
}

impl VkAsFieldsCommand {
    pub(crate) fn run(
        self,
        binary_path: &Path,
    ) -> Result<(FieldElement, Vec<FieldElement>), BackendError> {
        let mut command = std::process::Command::new(binary_path);

        command.arg("vk_as_fields").arg("-k").arg(self.vk_path).arg("-o").arg("-");

        let output = command.output()?;
        if output.status.success() {
            let string_output = String::from_utf8(output.stdout).unwrap();
            let mut fields: Vec<FieldElement> = serde_json::from_str(&string_output)
                .map_err(|err| BackendError::CommandFailed(err.to_string()))?;

            // The first element of this vector is the hash of the verification key, we want to split that off.
            let hash = fields.remove(0);
            Ok((hash, fields))
        } else {
            Err(BackendError::CommandFailed(string_from_stderr(&output.stderr)))
        }
    }
}
