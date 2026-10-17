//! Integration tests for `nargo test` when the test results cannot be written to stdout.
//!
//! A failed write ends the run with a non-zero exit code, from whichever thread hits it first:
//! with `--format json` worker threads write each test's start and end, and with the other formats
//! the main thread writes everything. A broken pipe means nobody is reading, so it exits without
//! an error message; any other failure is reported on stderr.

#![cfg(target_os = "linux")]

use std::fs::OpenOptions;
use std::process::{Command, Stdio};

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

/// Creates a project with a few passing tests and returns a `nargo test --format <format>`
/// command for it. The `TempDir` must outlive the command.
fn nargo_test_command(format: &str) -> (TempDir, Command) {
    let test_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&test_dir).arg("new").arg("output_errors");
    cmd.assert().success();

    let project_dir = test_dir.child("output_errors");
    project_dir.child("src").child("main.nr").write_str(SOURCE).unwrap();

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&project_dir).arg("test").arg("--format").arg(format);
    (test_dir, cmd)
}

/// Runs `nargo test --format <format>` with stdout connected to `/dev/full`, where every write
/// fails with `ENOSPC`.
fn run_with_full_stdout(format: &str) -> assert_cmd::assert::Assert {
    let (_test_dir, mut cmd) = nargo_test_command(format);
    let dev_full = OpenOptions::new().write(true).open("/dev/full").unwrap();
    cmd.stdout(dev_full);
    cmd.assert()
}

/// Runs `nargo test --format <format>` with stdout connected to a pipe whose read end is closed
/// before `nargo` writes anything, so every write fails with `EPIPE`.
fn run_with_closed_stdout_pipe(format: &str) -> assert_cmd::assert::Assert {
    let (_test_dir, mut cmd) = nargo_test_command(format);
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    drop(child.stdout.take());
    let output = child.wait_with_output().unwrap();
    assert_cmd::assert::Assert::new(output)
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

fn assert_fails_quietly_on_broken_pipe(format: &str) {
    run_with_closed_stdout_pipe(format)
        .code(1)
        .stderr(predicate::str::contains("Could not display test results").not())
        .stderr(predicate::str::contains("panicked").not());
}

#[test]
fn pretty_format_fails_quietly_on_broken_pipe() {
    assert_fails_quietly_on_broken_pipe("pretty");
}

#[test]
fn terse_format_fails_quietly_on_broken_pipe() {
    assert_fails_quietly_on_broken_pipe("terse");
}

#[test]
fn json_format_fails_quietly_on_broken_pipe() {
    assert_fails_quietly_on_broken_pipe("json");
}
