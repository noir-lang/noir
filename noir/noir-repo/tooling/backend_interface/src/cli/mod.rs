// Reference: https://github.com/AztecProtocol/aztec-packages/blob/master/barretenberg/cpp/src/barretenberg/bb/main.cpp

mod contract;
mod gates;
mod info;
mod proof_as_fields;
mod prove;
mod verify;
mod version;
mod vk_as_fields;
mod write_vk;

pub(crate) use contract::ContractCommand;
pub(crate) use gates::GatesCommand;
pub(crate) use info::InfoCommand;
pub(crate) use proof_as_fields::ProofAsFieldsCommand;
pub(crate) use prove::ProveCommand;
pub(crate) use verify::VerifyCommand;
pub(crate) use version::VersionCommand;
pub(crate) use vk_as_fields::VkAsFieldsCommand;
pub(crate) use write_vk::WriteVkCommand;

#[test]
fn no_command_provided_works() -> Result<(), crate::BackendError> {
    // This is a simple test to check that the binaries work

    let backend = crate::get_mock_backend()?;

    let output = std::process::Command::new(backend.binary_path()).output()?;

    let stderr = string_from_stderr(&output.stderr);
    // Assert help message is printed due to no command being provided.
    assert!(stderr.contains("Usage: mock_backend <COMMAND>"));

    Ok(())
}

// Converts a stderr byte array to a string (including invalid characters)
fn string_from_stderr(stderr: &[u8]) -> String {
    String::from_utf8_lossy(stderr).to_string()
}
