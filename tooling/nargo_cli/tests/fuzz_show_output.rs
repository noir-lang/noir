//! Integration test for `nargo test --show-output` on parameterized (fuzzed) tests.
//!
//! A test with arguments is executed by the greybox fuzzer rather than a single run. Its
//! `println` output must still be surfaced under `--show-output` (for a single representative
//! execution), matching the behavior of tests without arguments. See issue #8194.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

use assert_fs::TempDir;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::{FileWriteStr, PathChild};

/// A parameterized test prints a constant marker, so the assertion is deterministic regardless of
/// which input the fuzzer settles on.
const MAIN_WITH_FUZZED_TEST: &str = "\
fn main() {}

#[test]
fn prints(_x: u8) {
    println(\"fuzz output marker\");
}
";

fn new_project(test_dir: &TempDir, project_name: &str) -> ChildPath {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(test_dir).arg("new").arg(project_name);
    cmd.assert().success();

    let project_dir = test_dir.child(project_name);
    project_dir.child("src").child("main.nr").write_str(MAIN_WITH_FUZZED_TEST).unwrap();
    project_dir
}

#[test]
fn show_output_prints_output_for_parameterized_test() {
    let test_dir = TempDir::new().unwrap();
    let project_dir = new_project(&test_dir, "fuzz_show_output");

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&project_dir).arg("test").arg("--show-output").arg("--exact").arg("prints");
    cmd.assert().success().stdout(predicate::str::contains("fuzz output marker"));
}

#[test]
fn without_show_output_no_output_for_parameterized_test() {
    let test_dir = TempDir::new().unwrap();
    let project_dir = new_project(&test_dir, "fuzz_no_show_output");

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&project_dir).arg("test").arg("--exact").arg("prints");
    cmd.assert().success().stdout(predicate::str::contains("fuzz output marker").not());
}
