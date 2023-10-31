use std::path::{Path, PathBuf};

use acvm::FieldElement;

use crate::BackendError;

use super::string_from_stderr;

/// `ProofAsFieldsCommand` will call the barretenberg binary
/// to split a proof into a representation as [`FieldElement`]s.
pub(crate) struct ProofAsFieldsCommand {
    pub(crate) proof_path: PathBuf,
    pub(crate) vk_path: PathBuf,
}

impl ProofAsFieldsCommand {
    pub(crate) fn run(self, binary_path: &Path) -> Result<Vec<FieldElement>, BackendError> {
        let mut command = std::process::Command::new(binary_path);

        command
            .arg("proof_as_fields")
            .arg("-p")
            .arg(self.proof_path)
            .arg("-k")
            .arg(self.vk_path)
            .arg("-o")
            .arg("-");

        let output = command.output()?;
        if output.status.success() {
            let string_output = String::from_utf8(output.stdout).unwrap();
            serde_json::from_str(&string_output)
                .map_err(|err| BackendError::CommandFailed(err.to_string()))
        } else {
            Err(BackendError::CommandFailed(string_from_stderr(&output.stderr)))
        }
    }
}
