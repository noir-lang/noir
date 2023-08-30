use super::{assert_binary_exists, get_binary_path};

/// ProveAndVerifyCommand will call the barretenberg binary
/// to create a proof and then verify the proof once created.
///
/// Note: Functions like this are useful for testing. In a real workflow,
/// ProveCommand and VerifyCommand will be used separately.
#[allow(dead_code)]
struct ProveAndVerifyCommand {
    verbose: bool,
    path_to_crs: String,
    is_recursive: bool,
    path_to_bytecode: String,
    path_to_witness: String,
}

#[allow(dead_code)]
impl ProveAndVerifyCommand {
    fn run(self) -> bool {
        assert_binary_exists();
        let mut command = std::process::Command::new(get_binary_path());

        command
            .arg("prove_and_verify")
            .arg("-c")
            .arg(self.path_to_crs)
            .arg("-b")
            .arg(self.path_to_bytecode)
            .arg("-w")
            .arg(self.path_to_witness);
        if self.verbose {
            command.arg("-v");
        }
        if self.is_recursive {
            command.arg("-r");
        }

        command
            .output()
            .expect("Failed to execute command")
            .status
            .success()
    }
}

#[test]
fn prove_and_verify_command() {
    use tempfile::tempdir;

    let path_to_1_mul = "./src/1_mul.bytecode";
    let path_to_1_mul_witness = "./src/witness.tr";

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();
    let path_to_crs = temp_directory_path.join("crs");

    let prove_and_verify_command = ProveAndVerifyCommand {
        verbose: true,
        path_to_crs: path_to_crs.to_str().unwrap().to_string(),
        is_recursive: false,
        path_to_bytecode: path_to_1_mul.to_string(),
        path_to_witness: path_to_1_mul_witness.to_string(),
    };

    let output = prove_and_verify_command.run();
    assert!(output);
    drop(temp_directory);
}
