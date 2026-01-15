use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::path::PathBuf;
use std::process::Command;

/// Get noir-inspector command (binary is automatically built by cargo test)
fn inspector_command() -> Command {
    #[allow(deprecated)]
    Command::cargo_bin("noir-inspector").unwrap()
}

/// Get test program directory
fn test_program_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test_programs/execution_success/assert_statement")
}

/// Get test program artifact path, compiling if needed
fn test_artifact_path() -> PathBuf {
    let program_dir = test_program_dir();
    let artifact = program_dir.join("target/assert_statement.json");

    // Compile if artifact doesn't exist
    if !artifact.exists() {
        #[allow(deprecated)]
        let mut nargo = Command::cargo_bin("nargo").unwrap();
        nargo
            .arg("--program-dir")
            .arg(&program_dir)
            .arg("compile")
            .arg("--force")
            .arg("--force-brillig");

        nargo.assert().success();
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
    let input_file = test_program_dir().join("Prover.toml");

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
