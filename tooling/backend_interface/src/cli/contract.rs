use std::path::{Path, PathBuf};

use crate::BackendError;

use super::string_from_stderr;

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

        let output = command.output()?;

        if output.status.success() {
            String::from_utf8(output.stdout)
                .map_err(|error| BackendError::InvalidUTF8Vector(error.into_bytes()))
        } else {
            Err(BackendError::CommandFailed(string_from_stderr(&output.stderr)))
        }
    }
}

#[test]
fn contract_command() -> Result<(), BackendError> {
    use tempfile::tempdir;

    let backend = crate::get_mock_backend()?;

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();
    let bytecode_path = temp_directory_path.join("acir.gz");
    let vk_path = temp_directory_path.join("vk");

    let crs_path = backend.backend_directory();

    std::fs::File::create(&bytecode_path).expect("file should be created");

    let write_vk_command = super::WriteVkCommand {
        bytecode_path,
        vk_path_output: vk_path.clone(),
        crs_path: crs_path.clone(),
    };
    write_vk_command.run(backend.binary_path())?;

    let contract_command = ContractCommand { vk_path, crs_path };
    contract_command.run(backend.binary_path())?;

    drop(temp_directory);

    Ok(())
}
