//! Integration tests for `nargo clean`, which removes the target directory.

use assert_cmd::prelude::*;
use assert_fs::prelude::{FileWriteStr, PathAssert, PathChild};
use predicates::prelude::*;
use std::process::Command;

#[test]
fn clean_removes_the_target_directory() {
    let test_dir = assert_fs::TempDir::new().unwrap();
    let project_dir = test_dir.child("clean_me");

    // `nargo new clean_me`
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("new").arg(project_dir.path());
    cmd.assert().success();

    // Populate a target directory directly rather than compiling: `clean` only
    // cares that the directory exists, so this keeps the test cheap.
    let artifact = project_dir.child("target").child("clean_me.json");
    artifact.write_str("{}").unwrap();
    artifact.assert(predicate::path::is_file());

    // `nargo clean`
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(project_dir.path()).arg("clean");
    cmd.assert().success();

    project_dir.child("target").assert(predicate::path::missing());
    project_dir.child("src").assert(predicate::path::is_dir());
}

#[test]
fn clean_without_a_target_directory_is_a_no_op() {
    let test_dir = assert_fs::TempDir::new().unwrap();
    let project_dir = test_dir.child("never_built");

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("new").arg(project_dir.path());
    cmd.assert().success();

    // Cleaning a project which was never built should succeed and leave the
    // project directory itself untouched.
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(project_dir.path()).arg("clean");
    cmd.assert().success();
    project_dir.child("target").assert(predicate::path::missing());
    project_dir.child("src").assert(predicate::path::is_dir());
}
