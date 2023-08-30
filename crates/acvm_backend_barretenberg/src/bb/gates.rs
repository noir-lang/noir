use super::{assert_binary_exists, get_binary_path};

/// GatesCommand will call the barretenberg binary
/// to return the number of gates needed to create a proof
/// for the given bytecode.
pub(crate) struct GatesCommand {
    pub(crate) path_to_crs: String,
    pub(crate) path_to_bytecode: String,
}

impl GatesCommand {
    pub(crate) fn run(self) -> u32 {
        assert_binary_exists();
        let output = std::process::Command::new(get_binary_path())
            .arg("gates")
            .arg("-c")
            .arg(self.path_to_crs)
            .arg("-b")
            .arg(self.path_to_bytecode)
            .output()
            .expect("Failed to execute command");

        if !output.status.success() {
            panic!(
                "gates command encountered an error: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        // Note: barretenberg includes the newline, so that subsequent prints to stdout
        // are not on the same line as the gates output.

        // Ensure we got the expected number of bytes
        if output.stdout.len() != 8 {
            panic!("Unexpected 8 bytes, received {}", output.stdout.len());
        }

        // Convert bytes to u64 in little-endian format
        let value = u64::from_le_bytes([
            output.stdout[0],
            output.stdout[1],
            output.stdout[2],
            output.stdout[3],
            output.stdout[4],
            output.stdout[5],
            output.stdout[6],
            output.stdout[7],
        ]);

        value as u32
    }
}

#[test]
fn gate_command() {
    use tempfile::tempdir;

    let path_to_1_mul = "./src/1_mul.bytecode";

    let temp_directory = tempdir().expect("could not create a temporary directory");
    let temp_directory_path = temp_directory.path();
    let path_to_crs = temp_directory_path.join("crs");

    let gate_command = GatesCommand {
        path_to_crs: path_to_crs.to_str().unwrap().to_string(),
        path_to_bytecode: path_to_1_mul.to_string(),
    };

    let output = gate_command.run();
    assert_eq!(output, 2775);
    drop(temp_directory);
}
