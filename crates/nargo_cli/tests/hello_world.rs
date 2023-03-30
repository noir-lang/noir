//! This integration test aims to mirror the steps taken by a new user using Nargo for the first time.
//! It then follows the steps published at https://noir-lang.org/getting_started/hello_world.html
//! Any modifications to the commands run here MUST be documented in the noir-lang book.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

use assert_fs::prelude::{FileWriteStr, PathAssert, PathChild};

#[test]
fn hello_world_example() {
    let test_dir = assert_fs::TempDir::new().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    let project_name = "hello_world";
    let project_dir = test_dir.child(project_name);

    // `nargo new hello_world`
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("new").arg(project_name);
    cmd.assert().success().stdout(predicate::str::contains("Project successfully created!"));

    project_dir.child("src").assert(predicate::path::is_dir());
    project_dir.child("Nargo.toml").assert(predicate::path::is_file());

    std::env::set_current_dir(&project_dir).unwrap();

    // `nargo check`
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("check");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Constraint system successfully built!"));

    project_dir.child("Prover.toml").assert(predicate::path::is_file());
    project_dir.child("Verifier.toml").assert(predicate::path::is_file());

    // `nargo prove p`
    let proof_name = "p";
    project_dir.child("Prover.toml").write_str("x = 1\ny = 2").unwrap();

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("prove").arg(proof_name);
    cmd.assert().success();

    project_dir
        .child("proofs")
        .child(format!("{proof_name}.proof"))
        .assert(predicate::path::is_file());

    // `nargo verify p`
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("verify").arg(proof_name);
    cmd.assert().success();
}
