use std::path::{Path, PathBuf};

/// VerifyCommand will call the barretenberg binary
/// to verify a proof
pub(crate) struct VerifyCommand {
    pub(crate) crs_path: PathBuf,
    pub(crate) is_recursive: bool,
    pub(crate) proof_path: PathBuf,
    pub(crate) vk_path: PathBuf,
}

impl VerifyCommand {
    pub(crate) fn run(self, binary_path: &Path) -> bool {
        let mut command = std::process::Command::new(binary_path);

        command
            .arg("verify")
            .arg("-c")
            .arg(self.crs_path)
            .arg("-p")
            .arg(self.proof_path)
            .arg("-k")
            .arg(self.vk_path);

        if self.is_recursive {
            command.arg("-r");
        }

        let output = command.output().expect("Failed to execute command");

        // We currently do not distinguish between an invalid proof and an error inside the backend.
        output.status.success()
    }
}

#[test]
#[serial_test::serial]
fn verify_command() {
    use tempfile::tempdir;

    use super::{ProveCommand, WriteVkCommand};
    use crate::proof_system::write_to_file;

    let backend = crate::get_mock_backend();

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();
    let bytecode_path = temp_directory_path.join("acir.gz");
    let witness_path = temp_directory_path.join("witness.tr");
    let proof_path = temp_directory_path.join("1_mul.proof");
    let vk_path_output = temp_directory_path.join("vk");

    let crs_path = backend.backend_directory();

    std::fs::File::create(&bytecode_path).expect("file should be created");
    std::fs::File::create(&witness_path).expect("file should be created");

    let write_vk_command = WriteVkCommand {
        bytecode_path: bytecode_path.clone(),
        crs_path: crs_path.clone(),
        is_recursive: false,
        vk_path_output: vk_path_output.clone(),
    };

    let vk_written = write_vk_command.run(&backend.binary_path());
    assert!(vk_written.is_ok());

    let prove_command = ProveCommand {
        crs_path: crs_path.clone(),
        is_recursive: false,
        bytecode_path,
        witness_path,
    };
    let proof = prove_command.run(&backend.binary_path()).unwrap();

    write_to_file(&proof, &proof_path);

    let verify_command =
        VerifyCommand { crs_path, is_recursive: false, proof_path, vk_path: vk_path_output };

    let verified = verify_command.run(&backend.binary_path());
    assert!(verified);
    drop(temp_directory);
}
