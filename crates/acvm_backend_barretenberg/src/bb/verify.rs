use std::path::PathBuf;

use super::{assert_binary_exists, get_binary_path};

/// VerifyCommand will call the barretenberg binary
/// to verify a proof
pub(crate) struct VerifyCommand {
    pub(crate) verbose: bool,
    pub(crate) crs_path: PathBuf,
    pub(crate) is_recursive: bool,
    pub(crate) proof_path: PathBuf,
    pub(crate) vk_path: PathBuf,
}

impl VerifyCommand {
    pub(crate) fn run(self) -> bool {
        assert_binary_exists();
        let mut command = std::process::Command::new(get_binary_path());

        command
            .arg("verify")
            .arg("-c")
            .arg(self.crs_path)
            .arg("-p")
            .arg(self.proof_path)
            .arg("-k")
            .arg(self.vk_path);

        if self.verbose {
            command.arg("-v");
        }
        if self.is_recursive {
            command.arg("-r");
        }

        let output = command.output().expect("Failed to execute command");
        output.status.success()
    }
}

#[test]
#[serial_test::serial]
fn verify_command() {
    use tempfile::tempdir;

    use crate::bb::{ProveCommand, WriteVkCommand};

    let bytecode_path = PathBuf::from("./src/1_mul.bytecode");
    let witness_path = PathBuf::from("./src/witness.tr");

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();

    let crs_path = temp_directory_path.join("crs");
    let proof_path = temp_directory_path.join("1_mul").with_extension("proof");
    let vk_path_output = temp_directory_path.join("vk");

    let write_vk_command = WriteVkCommand {
        verbose: true,
        bytecode_path: bytecode_path.clone(),
        crs_path: crs_path.clone(),
        is_recursive: false,
        vk_path_output: vk_path_output.clone(),
    };

    let vk_written = write_vk_command.run();
    assert!(vk_written.is_ok());

    let prove_command = ProveCommand {
        verbose: true,
        crs_path: crs_path.clone(),
        is_recursive: false,
        bytecode_path,
        witness_path,
        proof_path: proof_path.clone(),
    };
    prove_command.run().unwrap();

    let verify_command = VerifyCommand {
        verbose: true,
        crs_path,
        is_recursive: false,
        proof_path,
        vk_path: vk_path_output,
    };

    let verified = verify_command.run();
    assert!(verified);
    drop(temp_directory);
}
