use std::path::{Path, PathBuf};

use crate::BackendError;

/// VerifyCommand will call the barretenberg binary
/// to return a solidity library with the verification key
/// that can be used to verify proofs on-chain.
///
/// This does not return a Solidity file that is able
/// to verify a proof. See acvm_interop/contract.sol for the
/// remaining logic that is missing.
pub(crate) struct ContractCommand {
    pub(crate) crs_path: PathBuf,
    pub(crate) vk_path: PathBuf,
}

impl ContractCommand {
    pub(crate) fn run(self, binary_path: &Path) -> Result<String, BackendError> {
        let mut command = std::process::Command::new(binary_path);

        command
            .arg("contract")
            .arg("-c")
            .arg(self.crs_path)
            .arg("-k")
            .arg(self.vk_path)
            .arg("-o")
            .arg("-");

        let output = command.output().expect("Failed to execute command");

        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else {
            Err(BackendError(String::from_utf8(output.stderr).unwrap()))
        }
    }
}

#[test]
fn contract_command() {
    use tempfile::tempdir;

    let backend = crate::get_mock_backend();

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();
    let bytecode_path = temp_directory_path.join("acir.gz");
    let vk_path = temp_directory_path.join("vk");

    let crs_path = backend.backend_directory();

    std::fs::File::create(&bytecode_path).expect("file should be created");

    let write_vk_command = super::WriteVkCommand {
        bytecode_path,
        vk_path_output: vk_path.clone(),
        is_recursive: false,
        crs_path: crs_path.clone(),
    };

    assert!(write_vk_command.run(&backend.binary_path()).is_ok());

    let contract_command = ContractCommand { vk_path, crs_path };

    assert!(contract_command.run(&backend.binary_path()).is_ok());
    drop(temp_directory);
}
