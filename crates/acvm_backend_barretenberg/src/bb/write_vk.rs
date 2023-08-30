use super::{assert_binary_exists, get_binary_path, CliShimError};

/// WriteCommand will call the barretenberg binary
/// to write a verification key to a file
pub(crate) struct WriteVkCommand {
    pub(crate) verbose: bool,
    pub(crate) path_to_crs: String,
    pub(crate) is_recursive: bool,
    pub(crate) path_to_bytecode: String,
    pub(crate) path_to_vk_output: String,
}

impl WriteVkCommand {
    pub(crate) fn run(self) -> Result<(), CliShimError> {
        assert_binary_exists();
        let mut command = std::process::Command::new(get_binary_path());

        command
            .arg("write_vk")
            .arg("-c")
            .arg(self.path_to_crs)
            .arg("-b")
            .arg(self.path_to_bytecode)
            .arg("-o")
            .arg(self.path_to_vk_output);

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
fn write_vk_command() {
    use tempfile::tempdir;

    let path_to_1_mul = "./src/1_mul.bytecode";

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();
    let path_to_crs = temp_directory_path.join("crs");
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
    drop(temp_directory);
}
