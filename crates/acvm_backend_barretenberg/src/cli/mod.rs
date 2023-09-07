// Reference: https://github.com/AztecProtocol/aztec-packages/blob/master/circuits/cpp/barretenberg/cpp/src/barretenberg/bb/main.cpp

mod contract;
mod gates;
mod info;
mod prove;
mod verify;
mod write_vk;

pub(crate) use contract::ContractCommand;
pub(crate) use gates::GatesCommand;
pub(crate) use info::InfoCommand;
pub(crate) use prove::ProveCommand;
pub(crate) use verify::VerifyCommand;
pub(crate) use write_vk::WriteVkCommand;

#[test]
#[serial_test::serial]
fn no_command_provided_works() {
    // This is a simple test to check that the binaries work

    let backend = crate::get_mock_backend();

    let output = std::process::Command::new(backend.binary_path())
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Assert help message is printed due to no command being provided.
    assert!(stderr.contains("Usage: mock_backend <COMMAND>"));
}
