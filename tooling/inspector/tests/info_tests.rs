use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::path::PathBuf;
use std::process::Command;

fn inspector_command() -> Command {
    #[allow(deprecated)]
    Command::cargo_bin("noir-inspector").unwrap()
}

fn test_program_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test_programs/execution_success/assert_statement")
}

/// get test program artifact path, always recompiling with --force-brillig
fn test_artifact_path() -> PathBuf {
    let program_dir = test_program_dir();
    let artifact = program_dir.join("target/assert_statement_brillig.json");

    // always recompile to ensure correct version
    #[allow(deprecated)]
    let mut nargo = Command::cargo_bin("nargo").unwrap();
    nargo
        .arg("--program-dir")
        .arg(&program_dir)
        .arg("compile")
        .arg("--force")
        .arg("--force-brillig");

    nargo.assert().success();

    let default_artifact = program_dir.join("target/assert_statement.json");
    std::fs::rename(&default_artifact, &artifact).unwrap();

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

/// get test program artifact compiled without --force-brillig
fn test_artifact_with_acir() -> PathBuf {
    let program_dir = test_program_dir();
    let artifact = program_dir.join("target/assert_statement_acir.json");

    #[allow(deprecated)]
    let mut nargo = Command::cargo_bin("nargo").unwrap();
    nargo.arg("--program-dir").arg(&program_dir).arg("compile").arg("--force");

    nargo.assert().success();

    let default_artifact = program_dir.join("target/assert_statement.json");
    std::fs::rename(&default_artifact, &artifact).unwrap();

    artifact
}

#[test]
fn test_profile_rejects_acir_programs() {
    let artifact = test_artifact_with_acir();
    let input_file = test_program_dir().join("Prover.toml");

    inspector_command()
        .arg("info")
        .arg(&artifact)
        .arg("--profile-execution")
        .arg("--input-file")
        .arg(&input_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot profile execution"))
        .stderr(predicate::str::contains("--force-brillig"));
}

#[test]
fn test_profile_accepts_pure_brillig() {
    // compiled with --force-brillig
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
