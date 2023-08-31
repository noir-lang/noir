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
    pub(crate) path_to_crs: String,
    pub(crate) is_recursive: bool,
    pub(crate) path_to_bytecode: String,
    pub(crate) path_to_witness: String,
    pub(crate) path_to_proof: String,
}

impl ProveCommand {
    pub(crate) fn run(self) -> Result<(), CliShimError> {
        assert_binary_exists();
        let mut command = std::process::Command::new(get_binary_path());

        command
            .arg("prove")
            .arg("-c")
            .arg(self.path_to_crs)
            .arg("-b")
            .arg(self.path_to_bytecode)
            .arg("-w")
            .arg(self.path_to_witness)
            .arg("-o")
            .arg(self.path_to_proof);

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

    let path_to_1_mul = "./src/1_mul.bytecode";
    let path_to_1_mul_witness = "./src/witness.tr";

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();

    let path_to_crs = temp_directory_path.join("crs");
    let path_to_proof = temp_directory_path.join("1_mul").with_extension("proof");

    let prove_command = ProveCommand {
        verbose: true,
        path_to_crs: path_to_crs.to_str().unwrap().to_string(),
        is_recursive: false,
        path_to_bytecode: path_to_1_mul.to_string(),
        path_to_witness: path_to_1_mul_witness.to_string(),
        path_to_proof: path_to_proof.to_str().unwrap().to_string(),
    };

    let proof_created = prove_command.run();
    assert!(proof_created.is_ok());
    drop(temp_directory);
}
