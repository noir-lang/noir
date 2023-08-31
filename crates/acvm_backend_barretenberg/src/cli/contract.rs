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
    pub(crate) verbose: bool,
    pub(crate) crs_path: PathBuf,
    pub(crate) vk_path: PathBuf,
    pub(crate) contract_path: PathBuf,
}

impl ContractCommand {
    pub(crate) fn run(self, binary_path: &Path) -> Result<(), BackendError> {
        let mut command = std::process::Command::new(binary_path);

        command
            .arg("contract")
            .arg("-c")
            .arg(self.crs_path)
            .arg("-k")
            .arg(self.vk_path)
            .arg("-o")
            .arg(self.contract_path);

        if self.verbose {
            command.arg("-v");
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
#[serial_test::serial]
fn contract_command() {
    use tempfile::tempdir;

    let bytecode_path = PathBuf::from("./src/1_mul.bytecode");

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();
    let crs_path = temp_directory_path.join("crs");
    let vk_path = temp_directory_path.join("vk");
    let contract_path = temp_directory_path.join("contract");

    let write_vk_command = super::WriteVkCommand {
        verbose: true,
        bytecode_path,
        vk_path_output: vk_path.clone(),
        is_recursive: false,
        crs_path: crs_path.clone(),
    };

    let binary_path = crate::assert_binary_exists();
    assert!(write_vk_command.run(&binary_path).is_ok());

    let contract_command = ContractCommand { verbose: true, vk_path, crs_path, contract_path };

    assert!(contract_command.run(&binary_path).is_ok());
    drop(temp_directory);
}
