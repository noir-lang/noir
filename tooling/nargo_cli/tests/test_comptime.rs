//! Forced comptime tests must take the same runtime branches as compiled tests.

use std::process::Command;

use assert_cmd::prelude::*;
use assert_fs::TempDir;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::{FileWriteStr, PathChild};
use predicates::prelude::*;

const RUNTIME_CONTEXT_SOURCE: &str = r#"
global COMPTIME_CONTEXT: bool = std::runtime::is_unconstrained();

fn context() -> bool {
    std::runtime::is_unconstrained()
}

unconstrained fn unconstrained_context() {
    assert(context(), "unconstrained callee");
    let callback = || context();
    assert(callback(), "unconstrained callback");
}

#[test]
fn constrained_test() {
    assert(!context(), "constrained test");
    // Safety: checking which runtime context the callee sees.
    unsafe { unconstrained_context(); }
    assert(!context(), "constrained context restored after call");
    let callback = || context();
    assert(!callback(), "constrained callback");
    assert(COMPTIME_CONTEXT, "globals are evaluated at comptime");
    assert(comptime { context() }, "comptime blocks remain unconstrained");
}

#[test]
unconstrained fn unconstrained_test() {
    assert(context(), "unconstrained test");
    unconstrained_context();
}

fn fail_in_current_context() {
    if context() {
        assert(false, "Brillig branch");
    } else {
        assert(false, "ACIR branch");
    }
}

#[test(should_fail_with = "ACIR branch")]
fn constrained_failure() {
    fail_in_current_context();
}

#[test(should_fail_with = "Brillig branch")]
unconstrained fn unconstrained_failure() {
    fail_in_current_context();
}

"#;

const FUNCTION_TYPE_SOURCE: &str = r#"
fn context() -> bool {
    std::runtime::is_unconstrained()
}

fn with_argument(caller_context: bool) -> bool {
    assert(!caller_context, "arguments use caller context");
    context()
}

#[test]
fn coerced_calls() {
    let callback: unconstrained fn() -> bool = || context();
    let function: unconstrained fn(bool) -> bool = with_argument;
    // Safety: checking runtime boundaries introduced by function types.
    unsafe {
        assert(callback(), "callback coerced to unconstrained");
        assert(function(context()), "function coerced to unconstrained");
    }
    assert(!context(), "constrained context restored after indirect calls");
}
"#;

const DEFAULT_ASSERTION_SOURCE: &str = r#"
#[test(should_fail_with = "Failed constraint")]
fn default_assertion_message() {
    assert(false);
}

#[test(should_fail_with = "Failed constraint")]
fn default_assert_eq_message() {
    assert_eq(1, 2);
}
"#;

const FORCED_BRILLIG_SOURCE: &str = r#"
fn context() -> bool {
    std::runtime::is_unconstrained()
}

fn fail_in_current_context() {
    if context() {
        assert(false, "Brillig branch");
    } else {
        assert(false, "ACIR branch");
    }
}

#[test]
fn forced_context() {
    assert(context(), "forced Brillig context");
    let callback = || context();
    assert(callback(), "forced Brillig callback");
    assert(comptime { context() }, "comptime remains unconstrained");
}

#[test(should_fail_with = "Brillig branch")]
fn forced_failure() {
    fail_in_current_context();
}

#[test(should_fail_with = "Failed to solve brillig function")]
fn forced_default_message() {
    assert(std::hash::pedersen_commitment([27]).x == 0);
}
"#;

const BRILLIG_ASSERTION_SOURCE: &str = r#"
unconstrained fn fail_without_message() {
    // A runtime computation keeps this assertion from failing during constant folding.
    assert(std::hash::pedersen_commitment([27]).x == 0);
}

#[test(should_fail_with = "Failed to solve brillig function")]
unconstrained fn unconstrained_failure() {
    fail_without_message();
}

#[test(should_fail_with = "Failed to solve brillig function")]
fn failure_in_unsafe_call() {
    // Safety: testing an assertion failure inside an unconstrained callee.
    unsafe { fail_without_message(); }
}
"#;

const ASSERTION_MESSAGE_ERROR_SOURCE: &str = r#"
fn build_message() -> str<6> {
    assert(false, "message evaluation failed");
    "unused"
}

#[test(should_fail_with = "message evaluation failed")]
fn assertion_message_error() {
    assert(false, build_message());
}

#[test(should_fail_with = "message evaluation failed")]
fn assert_eq_message_error() {
    assert_eq(1, 2, build_message());
}

#[test(should_fail_with = "message evaluation failed")]
unconstrained fn unconstrained_message_error() {
    assert(false, build_message());
}
"#;

fn new_project(test_dir: &TempDir, project_name: &str, source: &str) -> ChildPath {
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(test_dir).args(["new", "--lib", project_name]);
    cmd.assert().success();

    let project_dir = test_dir.child(project_name);
    project_dir.child("src/lib.nr").write_str(source).unwrap();
    project_dir
}

fn assert_modes_pass(source: &str, expected_summary: &str, extra_args: &[&str]) {
    let test_dir = TempDir::new().unwrap();
    let project_dir = new_project(&test_dir, "runtime_context", source);

    for mode in [None, Some("--force-comptime"), Some("--coverage")] {
        let mut cmd = Command::cargo_bin("nargo").unwrap();
        cmd.current_dir(&project_dir).args(["test", "--test-threads=1"]);
        cmd.args(extra_args);
        if let Some(mode) = mode {
            cmd.arg(mode);
        }
        cmd.assert().success().stdout(predicate::str::contains(expected_summary));
    }
}

#[test]
fn interpreted_tests_honor_runtime_context() {
    assert_modes_pass(RUNTIME_CONTEXT_SOURCE, "4 tests passed", &[]);
}

#[test]
fn interpreted_tests_honor_unconstrained_function_types() {
    assert_modes_pass(FUNCTION_TYPE_SOURCE, "1 test passed", &[]);
}

#[test]
fn interpreted_tests_match_default_assertion_messages() {
    assert_modes_pass(DEFAULT_ASSERTION_SOURCE, "2 tests passed", &[]);
}

#[test]
fn interpreted_tests_honor_force_brillig() {
    assert_modes_pass(FORCED_BRILLIG_SOURCE, "3 tests passed", &["--force-brillig"]);
}

#[test]
fn interpreted_tests_match_brillig_assertion_messages() {
    assert_modes_pass(BRILLIG_ASSERTION_SOURCE, "2 tests passed", &[]);
}

#[test]
fn interpreted_tests_propagate_assertion_message_errors() {
    assert_modes_pass(ASSERTION_MESSAGE_ERROR_SOURCE, "3 tests passed", &[]);
}

#[test]
fn interpreted_tests_preserve_custom_assertion_messages() {
    let test_dir = TempDir::new().unwrap();
    let project_dir = new_project(
        &test_dir,
        "assertion_message",
        r#"
#[test(should_fail_with = "Failed constraint")]
fn wrong_custom_message() {
    assert(false, "custom failure message");
}
"#,
    );

    for mode in [None, Some("--force-comptime"), Some("--coverage")] {
        let mut cmd = Command::cargo_bin("nargo").unwrap();
        cmd.current_dir(&project_dir).arg("test");
        if let Some(mode) = mode {
            cmd.arg(mode);
        }
        cmd.assert()
            .failure()
            .stdout(predicate::str::contains("Test failed with the wrong message"))
            .stderr(predicate::str::contains("custom failure message"));
    }
}
