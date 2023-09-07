use std::path::{Path, PathBuf};

use crate::BackendError;

/// ProveCommand will call the barretenberg binary
/// to create a proof, given the witness and the bytecode.
///
/// Note:Internally barretenberg will create and discard the
/// proving key, so this is not returned.
///
/// The proof will be written to the specified output file.
pub(crate) struct ProveCommand {
    pub(crate) crs_path: PathBuf,
    pub(crate) is_recursive: bool,
    pub(crate) bytecode_path: PathBuf,
    pub(crate) witness_path: PathBuf,
    pub(crate) proof_path: PathBuf,
}

impl ProveCommand {
    pub(crate) fn run(self, binary_path: &Path) -> Result<(), BackendError> {
        let mut command = std::process::Command::new(binary_path);

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

        if self.is_recursive {
            command.arg("-r");
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
fn prove_command() {
    use tempfile::tempdir;

    let backend = crate::get_mock_backend();

    let bytecode_path = PathBuf::from("./src/1_mul.bytecode");
    let witness_path = PathBuf::from("./src/witness.tr");

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let proof_path = temp_directory.path().join("1_mul").with_extension("proof");

    let crs_path = backend.backend_directory();
    let prove_command =
        ProveCommand { crs_path, bytecode_path, witness_path, is_recursive: false, proof_path };

    let proof_created = prove_command.run(&backend.binary_path());
    assert!(proof_created.is_ok());
    drop(temp_directory);
}
