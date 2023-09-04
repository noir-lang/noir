use std::path::PathBuf;

use super::{assert_binary_exists, get_binary_path, CliShimError};

/// ProveCommand will call the barretenberg binary
/// to create a proof, given the witness and the bytecode.
///
/// Note:Internally barretenberg will create and discard the
/// proving key, so this is not returned.
///
/// The proof will be written to the specified output file.
pub(crate) struct ProveCommand {
    pub(crate) verbose: bool,
    pub(crate) crs_path: PathBuf,
    pub(crate) is_recursive: bool,
    pub(crate) bytecode_path: PathBuf,
    pub(crate) witness_path: PathBuf,
    pub(crate) proof_path: PathBuf,
}

impl ProveCommand {
    pub(crate) fn run(self) -> Result<(), CliShimError> {
        assert_binary_exists();
        let mut command = std::process::Command::new(get_binary_path());

        command
            .arg("prove")
            .arg("-c")
            .arg(self.crs_path)
            .arg("-b")
            .arg(self.bytecode_path)
            .arg("-w")
            .arg(self.witness_path)
            .arg("-o")
            .arg(self.proof_path);

        if self.verbose {
            command.arg("-v");
        }
        if self.is_recursive {
            command.arg("-r");
        }

        let output = command.output().expect("Failed to execute command");

        if output.status.success() {
            Ok(())
        } else {
            Err(CliShimError(String::from_utf8(output.stderr).unwrap()))
        }
    }
}

#[test]
#[serial_test::serial]
fn prove_command() {
    use tempfile::tempdir;

    let bytecode_path = PathBuf::from("./src/1_mul.bytecode");
    let witness_path = PathBuf::from("./src/witness.tr");

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();

    let crs_path = temp_directory_path.join("crs");
    let proof_path = temp_directory_path.join("1_mul").with_extension("proof");

    let prove_command = ProveCommand {
        verbose: true,
        crs_path,
        bytecode_path,
        witness_path,
        is_recursive: false,
        proof_path,
    };

    let proof_created = prove_command.run();
    assert!(proof_created.is_ok());
    drop(temp_directory);
}
