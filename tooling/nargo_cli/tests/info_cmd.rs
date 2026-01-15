#[cfg(test)]
mod tests {
    use assert_cmd::prelude::*;
    use std::path::PathBuf;
    use std::process::Command;

    fn get_test_program_dir() -> PathBuf {
        // Use an existing simple test program
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("test_programs/execution_success/assert_statement")
    }

    #[test]
    fn info_does_not_have_expression_width_column() {
        #[allow(deprecated)]
        let mut cmd = Command::cargo_bin("nargo").unwrap();
        cmd.arg("--program-dir").arg(get_test_program_dir()).arg("info").arg("--force");

        let output = cmd.output().unwrap();
        assert!(output.status.success());

        let stdout = String::from_utf8(output.stdout).unwrap();

        // "Expression Width" should NOT be in the output
        assert!(
            !stdout.contains("Expression Width"),
            "Table should NOT have 'Expression Width' column"
        );

        // verify expected columns are present
        assert!(stdout.contains("Package"));
        assert!(stdout.contains("Function"));
        assert!(stdout.contains("ACIR Opcodes"));
        assert!(stdout.contains("Brillig Opcodes"));
    }

    #[test]
    fn info_has_correct_opcode_count() {
        #[allow(deprecated)]
        let mut cmd = Command::cargo_bin("nargo").unwrap();
        cmd.arg("--program-dir")
            .arg(get_test_program_dir())
            .arg("info")
            .arg("--force")
            .arg("--json");

        let output = cmd.output().unwrap();
        assert!(output.status.success());

        let json: serde_json::Value =
            serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();

        let programs = json["programs"].as_array().unwrap();
        assert!(!programs.is_empty());

        let functions = programs[0]["functions"].as_array().unwrap();
        assert!(!functions.is_empty());

        // verify opcode count is valid
        let opcode_count = functions[0]["opcodes"].as_u64().unwrap();
        assert!(opcode_count > 0, "Should have at least 1 opcode");
        assert!(opcode_count < 100, "Simple program should have fewer than 100 opcodes");
    }
}
