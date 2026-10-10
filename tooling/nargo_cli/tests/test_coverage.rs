//! Integration tests for `nargo test --coverage`, which runs tests in the comptime interpreter
//! and writes an lcov report of the code they evaluated.

use std::process::Command;

use assert_cmd::prelude::*;
use assert_fs::TempDir;
use assert_fs::prelude::{FileWriteStr, PathChild};
use predicates::prelude::*;

const SOURCE: &str = r#"
fn double(x: Field) -> Field {
    x * 2
}

fn triple(x: Field) -> Field {
    x * 3
}

#[test]
fn test_double() {
    println("printed by test_double");
    assert_eq(double(2), 4);
}

#[test]
fn test_triple() {
    assert_eq(triple(2), 6);
}
"#;

/// Several tests share one elaborated package, so this covers each of them getting its own
/// output and its own coverage record.
#[test]
fn coverage_run_passes_shows_output_and_writes_a_record_per_test() {
    let test_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&test_dir).arg("new").arg("--lib").arg("coverage");
    cmd.assert().success();

    let project_dir = test_dir.child("coverage");
    project_dir.child("src").child("lib.nr").write_str(SOURCE).unwrap();

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    // A single thread runs both tests against the same context.
    cmd.current_dir(&project_dir).args(["test", "--coverage", "--show-output", "--test-threads=1"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("printed by test_double"))
        .stdout(predicate::str::contains("2 tests passed"));

    let report = project_dir.child("target").child("coverage").child("lcov.info");
    let report = std::fs::read_to_string(report.path()).expect("coverage report should exist");
    assert!(report.contains("TN:test_double"), "missing test_double record:\n{report}");
    assert!(report.contains("TN:test_triple"), "missing test_triple record:\n{report}");
}
