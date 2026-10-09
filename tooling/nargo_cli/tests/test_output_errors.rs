//! Integration tests for `nargo test` when the test results cannot be written to stdout.
//!
//! A failed write is reported as a CLI error with a non-zero exit code, from whichever thread
//! hits it first: worker threads write the `test_start_async`/`test_end_async` events and the
//! main thread writes the per-package start, status and report events.

#![cfg(target_os = "linux")]

use std::fs::OpenOptions;
use std::process::Command;

use assert_cmd::prelude::*;
use assert_fs::TempDir;
use assert_fs::prelude::{FileWriteStr, PathChild};
use predicates::prelude::*;

const SOURCE: &str = r#"
fn main() {}

#[test]
fn a() {}

#[test]
fn b() {}

#[test]
fn c() {}
"#;

/// Runs `nargo test --format <format>` with stdout connected to `/dev/full`, where every write
/// fails with `ENOSPC`.
fn run_with_full_stdout(format: &str) -> assert_cmd::assert::Assert {
    let test_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&test_dir).arg("new").arg("full_stdout");
    cmd.assert().success();

    let project_dir = test_dir.child("full_stdout");
    project_dir.child("src").child("main.nr").write_str(SOURCE).unwrap();

    let dev_full = OpenOptions::new().write(true).open("/dev/full").unwrap();
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&project_dir).arg("test").arg("--format").arg(format).stdout(dev_full);
    cmd.assert()
}

fn assert_reports_write_error(format: &str) {
    run_with_full_stdout(format)
        .code(1)
        .stderr(predicate::str::contains("Could not display test results"))
        .stderr(predicate::str::contains("panicked").not());
}

#[test]
fn pretty_format_reports_stdout_write_failure() {
    assert_reports_write_error("pretty");
}

#[test]
fn terse_format_reports_stdout_write_failure() {
    assert_reports_write_error("terse");
}

#[test]
fn json_format_reports_stdout_write_failure() {
    assert_reports_write_error("json");
}
