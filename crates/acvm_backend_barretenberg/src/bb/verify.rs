use super::{assert_binary_exists, get_binary_path};

/// VerifyCommand will call the barretenberg binary
/// to verify a proof
pub(crate) struct VerifyCommand {
    pub(crate) verbose: bool,
    pub(crate) path_to_crs: String,
    pub(crate) is_recursive: bool,
    pub(crate) path_to_proof: String,
    pub(crate) path_to_vk: String,
}

impl VerifyCommand {
    pub(crate) fn run(self) -> bool {
        assert_binary_exists();
        let mut command = std::process::Command::new(get_binary_path());

        command
            .arg("verify")
            .arg("-c")
            .arg(self.path_to_crs)
            .arg("-p")
            .arg(self.path_to_proof)
            .arg("-k")
            .arg(self.path_to_vk);

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

    let path_to_1_mul = "./src/1_mul.bytecode";
    let path_to_1_mul_witness = "./src/witness.tr";

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();

    let path_to_crs = temp_directory_path.join("crs");
    let path_to_proof = temp_directory_path.join("1_mul").with_extension("proof");
    let path_to_vk = temp_directory_path.join("vk");

    let write_vk_command = WriteVkCommand {
        verbose: true,
        path_to_bytecode: path_to_1_mul.to_string(),
        path_to_crs: path_to_crs.to_str().unwrap().to_string(),
        is_recursive: false,
        path_to_vk_output: path_to_vk.to_str().unwrap().to_string(),
    };

    let vk_written = write_vk_command.run();
    assert!(vk_written.is_ok());

    let prove_command = ProveCommand {
        verbose: true,
        path_to_crs: path_to_crs.to_str().unwrap().to_string(),
        is_recursive: false,
        path_to_bytecode: path_to_1_mul.to_string(),
        path_to_witness: path_to_1_mul_witness.to_string(),
        path_to_proof: path_to_proof.to_str().unwrap().to_string(),
    };
    prove_command.run().unwrap();

    let verify_command = VerifyCommand {
        verbose: true,
        path_to_crs: path_to_crs.to_str().unwrap().to_string(),
        is_recursive: false,
        path_to_proof: path_to_proof.to_str().unwrap().to_string(),
        path_to_vk: path_to_vk.to_str().unwrap().to_string(),
    };

    let verified = verify_command.run();
    assert!(verified);
    drop(temp_directory);
}
