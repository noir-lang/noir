use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::path::PathBuf;
use std::process::Command;

/// Path to the noir-inspector binary
fn inspector_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_noir-inspector"))
}

/// Path to a existing test program
fn test_artifact_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test_programs/execution_success/assert_statement/target/assert_statement.json")
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
