#[cfg(test)]
mod tests {
    use assert_cmd::prelude::*;
    use std::path::PathBuf;
    use std::process::Command;

    fn get_test_artifact() -> PathBuf {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let test_program_dir = PathBuf::from(manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("test_programs/execution_success/assert_statement");

        // Compile the test program first
        #[allow(deprecated)]
        let mut compile_cmd = Command::cargo_bin("nargo").unwrap();
        compile_cmd.arg("--program-dir").arg(&test_program_dir).arg("compile").arg("--force");

        let output = compile_cmd.output().unwrap();
        assert!(output.status.success(), "Failed to compile test program");

        test_program_dir.join("target/assert_statement.json")
    }

    #[test]
    fn inspector_info_does_not_have_expression_width_column() {
        let artifact_path = get_test_artifact();

        #[allow(deprecated)]
        let mut cmd = Command::cargo_bin("noir-inspector").unwrap();
        cmd.arg("info").arg(&artifact_path);

        let output = cmd.output().unwrap();
        assert!(output.status.success());

        let stdout = String::from_utf8(output.stdout).unwrap();

        // Main assertion: "Expression Width" should NOT be in the output
        assert!(
            !stdout.contains("Expression Width"),
            "Table should NOT have 'Expression Width' column"
        );

        // Verify expected columns are present
        assert!(stdout.contains("Package"));
        assert!(stdout.contains("Function"));
        assert!(stdout.contains("ACIR Opcodes"));
        assert!(stdout.contains("Brillig Opcodes"));
    }

    #[test]
    fn inspector_info_has_correct_opcode_count() {
        let artifact_path = get_test_artifact();

        #[allow(deprecated)]
        let mut cmd = Command::cargo_bin("noir-inspector").unwrap();
        cmd.arg("info").arg(&artifact_path).arg("--json");

        let output = cmd.output().unwrap();
        assert!(output.status.success());

        let json: serde_json::Value =
            serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();

        let programs = json["programs"].as_array().unwrap();
        let functions = programs[0]["functions"].as_array().unwrap();

        // Verify concrete opcode count
        let opcode_count = functions[0]["opcodes"].as_u64().unwrap();
        assert!(opcode_count > 0, "Should have at least 1 opcode");
        assert!(opcode_count < 100, "Simple program should have fewer than 100 opcodes");
    }
}
