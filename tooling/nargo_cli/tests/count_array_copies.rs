//! Integration tests for the `--count-array-copies` debugging flag, which is exposed only on
//! `execute` and must compile and run in memory without persisting an artifact (its Brillig
//! instrumentation would otherwise poison the artifact cache).

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

use assert_fs::TempDir;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::{FileWriteStr, PathAssert, PathChild};

/// A program whose unconstrained code copies arrays, so `--count-array-copies` has something
/// to report (the copy counter only runs inside Brillig).
const MAIN_WITH_UNCONSTRAINED_COPY: &str = "\
unconstrained fn build(x: u64) -> u64 {
    let mut a = [x; 4];
    let mut b = a;
    b[0] = x + 1;
    a[1] = b[0];
    a[0]
}

fn main(x: u64, y: pub u64) {
    // Safety: test program
    let r = unsafe { build(x) };
    assert(r == x);
    assert(x != y);
}
";

/// Create a fresh package inside `test_dir` and return its directory.
fn new_project(test_dir: &TempDir, project_name: &str) -> ChildPath {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(test_dir).arg("new").arg(project_name);
    cmd.assert().success();

    let project_dir = test_dir.child(project_name);
    project_dir.child("src").child("main.nr").write_str(MAIN_WITH_UNCONSTRAINED_COPY).unwrap();
    project_dir.child("Prover.toml").write_str("x = 1\ny = 2").unwrap();
    project_dir
}

#[test]
fn execute_with_count_array_copies_does_not_generate_artifact() {
    let test_dir = TempDir::new().unwrap();
    let project_name = "count_copies";
    let project_dir = new_project(&test_dir, project_name);

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&project_dir).arg("execute").arg("--count-array-copies");
    cmd.assert().success().stdout(predicate::str::contains("Total arrays copied"));

    // No compilation artifact should have been written to the target directory.
    project_dir
        .child("target")
        .child(format!("{project_name}.json"))
        .assert(predicate::path::missing());
}

#[test]
fn compile_rejects_count_array_copies() {
    let test_dir = TempDir::new().unwrap();
    let project_dir = new_project(&test_dir, "count_copies_compile");

    // The flag is exposed only on `execute` (its Brillig instrumentation must never be persisted
    // to an artifact), so `compile` must reject it as an unknown argument.
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&project_dir).arg("compile").arg("--count-array-copies");
    cmd.assert().failure().stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn execute_without_count_array_copies_generates_artifact() {
    let test_dir = TempDir::new().unwrap();
    let project_name = "count_copies_baseline";
    let project_dir = new_project(&test_dir, project_name);

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&project_dir).arg("execute");
    cmd.assert().success();

    // A plain `execute` does persist the compilation artifact.
    project_dir
        .child("target")
        .child(format!("{project_name}.json"))
        .assert(predicate::path::is_file());
}
