use std::path::{Path, PathBuf};

use crate::BackendError;

use super::string_from_stderr;

/// ProveCommand will call the barretenberg binary
/// to create a proof, given the witness and the bytecode.
///
/// Note:Internally barretenberg will create and discard the
/// proving key, so this is not returned.
///
/// The proof will be written to the specified output file.
pub(crate) struct ProveCommand {
    pub(crate) crs_path: PathBuf,
    pub(crate) bytecode_path: PathBuf,
    pub(crate) witness_path: PathBuf,
}

impl ProveCommand {
    pub(crate) fn run(self, binary_path: &Path) -> Result<Vec<u8>, BackendError> {
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
            .arg("-");

        let output = command.output()?;
        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(BackendError::CommandFailed(string_from_stderr(&output.stderr)))
        }
    }
}

#[test]
fn prove_command() -> Result<(), BackendError> {
    use tempfile::tempdir;

    let backend = crate::get_mock_backend()?;

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();
    let bytecode_path = temp_directory_path.join("acir.gz");
    let witness_path = temp_directory_path.join("witness.tr");

    std::fs::File::create(&bytecode_path).expect("file should be created");
    std::fs::File::create(&witness_path).expect("file should be created");

    let crs_path = backend.backend_directory();
    let prove_command = ProveCommand { crs_path, bytecode_path, witness_path };

    let proof = prove_command.run(backend.binary_path())?;
    assert_eq!(proof, "proof".as_bytes());
    drop(temp_directory);

    Ok(())
}
