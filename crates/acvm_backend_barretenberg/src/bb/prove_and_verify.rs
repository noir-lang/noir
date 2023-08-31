use std::path::PathBuf;

use super::{assert_binary_exists, get_binary_path};

/// ProveAndVerifyCommand will call the barretenberg binary
/// to create a proof and then verify the proof once created.
///
/// Note: Functions like this are useful for testing. In a real workflow,
/// ProveCommand and VerifyCommand will be used separately.
#[allow(dead_code)]
struct ProveAndVerifyCommand {
    verbose: bool,
    crs_path: PathBuf,
    is_recursive: bool,
    bytecode_path: PathBuf,
    witness_path: PathBuf,
}

#[allow(dead_code)]
impl ProveAndVerifyCommand {
    fn run(self) -> bool {
        assert_binary_exists();
        let mut command = std::process::Command::new(get_binary_path());

        command
            .arg("prove_and_verify")
            .arg("-c")
            .arg(self.crs_path)
            .arg("-b")
            .arg(self.bytecode_path)
            .arg("-w")
            .arg(self.witness_path);
        if self.verbose {
            command.arg("-v");
        }
        if self.is_recursive {
            command.arg("-r");
        }

        command.output().expect("Failed to execute command").status.success()
    }
}

#[test]
#[serial_test::serial]
fn prove_and_verify_command() {
    use tempfile::tempdir;

    let bytecode_path = PathBuf::from("./src/1_mul.bytecode");
    let witness_path = PathBuf::from("./src/witness.tr");

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();
    let crs_path = temp_directory_path.join("crs");

    let prove_and_verify_command = ProveAndVerifyCommand {
        verbose: true,
        crs_path,
        is_recursive: false,
        bytecode_path,
        witness_path,
    };

    let output = prove_and_verify_command.run();
    assert!(output);
    drop(temp_directory);
}
