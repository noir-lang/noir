use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::path::PathBuf;
use std::process::Command;

/// Path to the noir-inspector binary, building it if needed
fn inspector_command() -> Command {
    let bin_path = env!("CARGO_BIN_EXE_noir-inspector");
    let bin = std::path::Path::new(bin_path);

    if !bin.exists() {
        Command::new("cargo")
            .args(["build", "-p", "noir_inspector", "--bin", "noir-inspector"])
            .output()
            .expect("Failed to build noir-inspector");

        assert!(bin.exists(), "Binary still doesn't exist after building: {bin:?}");
    }

    Command::new(bin_path)
}

/// Get test program artifact path, compiling it if needed
fn test_artifact_path() -> PathBuf {
    let program_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test_programs/execution_success/assert_statement");
    let artifact = program_dir.join("target/assert_statement.json");

    // Compile if artifact doesn't exist
    if !artifact.exists() {
        let output = Command::new("cargo")
            .args(["run", "-p", "nargo_cli", "--bin", "nargo", "--"])
            .arg("compile")
            .arg("--force-brillig")
            .current_dir(&program_dir)
            .output()
            .expect("Failed to run nargo compile");

        if !output.status.success() {
            panic!(
                "Failed to compile test program:\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        assert!(artifact.exists(), "Artifact still doesn't exist after compilation: {artifact:?}");
    }

    artifact
}

#[test]
fn test_profile_execution() {
    let artifact = test_artifact_path();

    inspector_command()
        .arg("info")
        .arg(&artifact)
        .arg("--profile-execution")
        .assert()
        .success()
        .stdout(predicate::str::contains("assert_statement"));
}

#[test]
fn test_profile_execution_with_explicit_input() {
    let artifact = test_artifact_path();
    let input_file = artifact.parent().unwrap().parent().unwrap().join("Prover.toml");

    inspector_command()
        .arg("info")
        .arg(&artifact)
        .arg("--profile-execution")
        .arg("--input-file")
        .arg(&input_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("assert_statement"));
}

#[test]
fn test_profile_execution_input_not_found() {
    let artifact = test_artifact_path();

    inspector_command()
        .arg("info")
        .arg(&artifact)
        .arg("--profile-execution")
        .arg("--input-file")
        .arg("/nonexistent/input.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Input file not found"));
}

#[test]
fn test_help_shows_new_options() {
    inspector_command()
        .arg("info")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("profile-execution"))
        .stdout(predicate::str::contains("input-file"))
        .stdout(predicate::str::contains("pedantic-solving"));
}
