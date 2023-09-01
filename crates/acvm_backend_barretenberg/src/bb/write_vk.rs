use std::path::PathBuf;

use super::{assert_binary_exists, get_binary_path, CliShimError};

/// WriteCommand will call the barretenberg binary
/// to write a verification key to a file
pub(crate) struct WriteVkCommand {
    pub(crate) verbose: bool,
    pub(crate) crs_path: PathBuf,
    pub(crate) is_recursive: bool,
    pub(crate) bytecode_path: PathBuf,
    pub(crate) vk_path_output: PathBuf,
}

impl WriteVkCommand {
    pub(crate) fn run(self) -> Result<(), CliShimError> {
        assert_binary_exists();
        let mut command = std::process::Command::new(get_binary_path());

        command
            .arg("write_vk")
            .arg("-c")
            .arg(self.crs_path)
            .arg("-b")
            .arg(self.bytecode_path)
            .arg("-o")
            .arg(self.vk_path_output);

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
fn write_vk_command() {
    use tempfile::tempdir;

    let bytecode_path = PathBuf::from("./src/1_mul.bytecode");

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();
    let crs_path = temp_directory_path.join("crs");
    let vk_path_output = temp_directory_path.join("vk");

    let write_vk_command = WriteVkCommand {
        verbose: true,
        bytecode_path,
        crs_path,
        is_recursive: false,
        vk_path_output,
    };

    let vk_written = write_vk_command.run();
    assert!(vk_written.is_ok());
    drop(temp_directory);
}
