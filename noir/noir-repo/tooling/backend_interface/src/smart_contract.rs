use std::path::PathBuf;

use crate::{
    cli::{ContractCommand, WriteVkCommand},
    Backend, BackendError,
};
use tempfile::tempdir;

impl Backend {
    pub fn eth_contract(&self, artifact_path: PathBuf) -> Result<String, BackendError> {
        let binary_path = self.assert_binary_exists()?;
        self.assert_correct_version()?;

        let temp_directory = tempdir().expect("could not create a temporary directory");
        let temp_directory_path = temp_directory.path().to_path_buf();

        // Create the verification key and write it to the specified path
        let vk_path = temp_directory_path.join("vk");

        WriteVkCommand {
            crs_path: self.crs_directory(),
            artifact_path,
            vk_path_output: vk_path.clone(),
        }
        .run(binary_path)?;

        ContractCommand { crs_path: self.crs_directory(), vk_path }.run(binary_path)
    }
}

#[cfg(test)]
mod tests {

    use serde_json::json;
    use tempfile::tempdir;

    use crate::{get_mock_backend, proof_system::write_to_file, BackendError};

    #[test]
    fn test_smart_contract() -> Result<(), BackendError> {
        let dummy_artifact = json!({"bytecode": ""});
        let artifact_bytes = serde_json::to_vec(&dummy_artifact).unwrap();

        let temp_directory = tempdir().expect("could not create a temporary directory");
        let temp_directory_path = temp_directory.path();
        let artifact_path = temp_directory_path.join("program.json");
        write_to_file(&artifact_bytes, &artifact_path);

        let contract = get_mock_backend()?.eth_contract(artifact_path)?;

        assert!(contract.contains("contract VerifierContract"));

        Ok(())
    }
}
