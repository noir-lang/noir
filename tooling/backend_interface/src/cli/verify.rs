use std::path::{Path, PathBuf};

use crate::BackendError;

/// VerifyCommand will call the barretenberg binary
/// to verify a proof
pub(crate) struct VerifyCommand {
    pub(crate) crs_path: PathBuf,
    pub(crate) is_recursive: bool,
    pub(crate) proof_path: PathBuf,
    pub(crate) bytecode_path: PathBuf,
}

impl VerifyCommand {
    pub(crate) fn run(self, binary_path: &Path) -> Result<bool, BackendError> {
        let mut command = std::process::Command::new(binary_path);

        command
            .arg("verify")
            .arg("-c")
            .arg(self.crs_path)
            .arg("-b")
            .arg(self.bytecode_path)
            .arg("-p")
            .arg(self.proof_path);

        if self.is_recursive {
            command.arg("-r");
        }

        let output = command.output()?;

        // We currently do not distinguish between an invalid proof and an error inside the backend.
        Ok(output.status.success())
    }
}

#[test]
fn verify_command() -> Result<(), BackendError> {
    use tempfile::tempdir;

    use super::ProveCommand;
    use crate::proof_system::write_to_file;

    let backend = crate::get_mock_backend()?;

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();
    let bytecode_path = temp_directory_path.join("acir.gz");
    let witness_path = temp_directory_path.join("witness.tr");
    let proof_path = temp_directory_path.join("1_mul.proof");

    let crs_path = backend.backend_directory();

    std::fs::File::create(&bytecode_path).expect("file should be created");
    std::fs::File::create(&witness_path).expect("file should be created");

    let prove_command = ProveCommand {
        crs_path: crs_path.clone(),
        is_recursive: false,
        bytecode_path: bytecode_path.clone(),
        witness_path,
    };
    let proof = prove_command.run(backend.binary_path())?;

    write_to_file(&proof, &proof_path);

    let verify_command = VerifyCommand { crs_path, is_recursive: false, proof_path, bytecode_path };

    let verified = verify_command.run(backend.binary_path())?;
    assert!(verified);

    drop(temp_directory);
    Ok(())
}
