//! Integration tests for `nargo test` sharing one elaborated `Context` between the tests a worker
//! thread runs.
//!
//! Sharing is only sound while every test leaves the context as it found it, and monomorphization
//! does not: it force-binds type variables that live in the shared `NodeInterner` and unbinds them
//! only after walking the function it is compiling all the way through. A test whose compilation
//! fails therefore hands on a context with generics still bound to its own instantiation.
//!
//! The invariant asserted here is the one that has to hold whatever the compiler does internally:
//! a test's result must not depend on which tests ran before it on the same thread. Each case
//! compares a whole suite run on a single thread against the same tests run one process at a time.

use std::collections::BTreeMap;
use std::process::Command;

use assert_cmd::prelude::*;
use assert_fs::TempDir;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::{FileWriteStr, PathChild};

/// `a_transmute_mismatch` fails inside the generic `transmute_pair`, so the failure is raised part
/// way through monomorphizing it, while `transmute_pair`'s generics are bound to that call's
/// instantiation. `#[test(should_fail)]` with no expected message reports the failure as a pass,
/// which is what makes this shape worth pinning: the suite is all green either way, so a
/// difference can only be seen by comparing against the isolated runs.
const COMPILE_FAILURE_BETWEEN_TESTS: &str = r#"
fn main() {}

fn transmute_pair<T, U>(x: T) -> U {
    std::mem::checked_transmute(x)
}

#[test(should_fail)]
fn a_transmute_mismatch() {
    let _: Field = transmute_pair([1, 2, 3]);
}

#[test]
fn b_transmute_field() {
    let y: Field = transmute_pair(7);
    assert_eq(y, 7);
}

#[test]
fn c_transmute_u8() {
    let y: u8 = transmute_pair(9 as u8);
    assert_eq(y, 9);
}

#[test(should_fail)]
fn d_transmute_mismatch_again() {
    let _: u8 = transmute_pair([1, 2]);
}

#[test]
fn e_transmute_field_again() {
    let y: Field = transmute_pair(11);
    assert_eq(y, 11);
}
"#;

/// A suite whose tests only differ in which impl of a trait they drive. Monomorphizing a call site
/// writes the instantiation bindings it worked out back into the interner, so the second test
/// reaches a call site that the first has already written to.
const SHARED_TRAIT_CALL_SITE: &str = r#"
fn main() {}

trait Named {
    fn tag(self) -> Field;

    fn describe(self) -> Field {
        self.tag() + 100
    }
}

struct Foo {}
struct Bar {}
struct Baz {}

impl Named for Foo {
    fn tag(self) -> Field { 1 }
}
impl Named for Bar {
    fn tag(self) -> Field { 2 }
}
impl Named for Baz {
    fn tag(self) -> Field { 3 }
}

fn describe_it<T>(x: T) -> Field where T: Named {
    x.describe()
}

#[test]
fn a_foo() { assert_eq(describe_it(Foo {}), 101); }

#[test]
fn b_bar() { assert_eq(describe_it(Bar {}), 102); }

#[test]
fn c_baz() { assert_eq(describe_it(Baz {}), 103); }
"#;

fn new_project(test_dir: &TempDir, project_name: &str, source: &str) -> ChildPath {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(test_dir).arg("new").arg(project_name);
    cmd.assert().success();

    let project_dir = test_dir.child(project_name);
    project_dir.child("src").child("main.nr").write_str(source).unwrap();
    project_dir
}

/// Run `nargo test` in `project_dir` and return each test's name mapped to its reported event
/// (`ok`, `failed` or `ignored`).
fn test_results(project_dir: &ChildPath, extra_args: &[&str]) -> BTreeMap<String, String> {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(project_dir).arg("test").arg("--format").arg("json");
    cmd.args(extra_args);

    let output = cmd.output().expect("failed to run nargo test");
    let stdout = String::from_utf8(output.stdout).expect("nargo test output should be utf8");

    let mut results = BTreeMap::new();
    for line in stdout.lines() {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        // The suite-level events carry no `name`, which is what distinguishes them here.
        if event["type"] != "test" {
            continue;
        }
        let (Some(name), Some(status)) = (event["name"].as_str(), event["event"].as_str()) else {
            continue;
        };
        // Each test is announced when it starts and again when it finishes; only the outcome
        // is of interest here.
        if status == "started" {
            continue;
        }
        if let Some(previous) = results.insert(name.to_string(), status.to_string()) {
            panic!("test {name} reported twice: {previous} then {status}");
        }
    }
    assert!(!results.is_empty(), "no test results parsed from:\n{stdout}");
    results
}

/// Run each of `test_names` in its own `nargo test` process, so no test can be influenced by
/// another, and collect the results into one map.
fn isolated_results(project_dir: &ChildPath, test_names: &[&str]) -> BTreeMap<String, String> {
    let mut results = BTreeMap::new();
    for name in test_names {
        results.extend(test_results(project_dir, &["--exact", name]));
    }
    results
}

#[test]
fn compile_failure_does_not_change_later_tests_on_the_same_thread() {
    let test_dir = TempDir::new().unwrap();
    let project_dir =
        new_project(&test_dir, "reuse_compile_failure", COMPILE_FAILURE_BETWEEN_TESTS);

    let names = [
        "a_transmute_mismatch",
        "b_transmute_field",
        "c_transmute_u8",
        "d_transmute_mismatch_again",
        "e_transmute_field_again",
    ];

    // A single thread guarantees every test after the first compiles against a context an earlier
    // test already used, which is the arrangement the invariant is about.
    let shared = test_results(&project_dir, &["--test-threads", "1"]);
    assert_eq!(shared, isolated_results(&project_dir, &names));
}

#[test]
fn shared_trait_call_site_does_not_change_later_tests_on_the_same_thread() {
    let test_dir = TempDir::new().unwrap();
    let project_dir = new_project(&test_dir, "reuse_shared_trait", SHARED_TRAIT_CALL_SITE);

    let shared = test_results(&project_dir, &["--test-threads", "1"]);
    assert_eq!(shared, isolated_results(&project_dir, &["a_foo", "b_bar", "c_baz"]));
}

#[test]
fn no_context_reuse_gives_the_same_results_as_reuse() {
    let test_dir = TempDir::new().unwrap();
    let project_dir = new_project(&test_dir, "reuse_flag_parity", COMPILE_FAILURE_BETWEEN_TESTS);

    let reused = test_results(&project_dir, &["--test-threads", "1"]);
    let fresh = test_results(&project_dir, &["--test-threads", "1", "--no-context-reuse"]);
    assert_eq!(reused, fresh);
}

#[test]
fn skipped_tests_are_still_reported_when_the_fuzzing_flags_exclude_them() {
    let test_dir = TempDir::new().unwrap();
    let project_dir = new_project(&test_dir, "reuse_only_fuzz", SHARED_TRAIT_CALL_SITE);

    // Every test here is argument-free, so `--only-fuzz` excludes all of them. They must still be
    // reported rather than silently dropped, and excluding a test must not depend on having built
    // a context for it.
    let results = test_results(&project_dir, &["--only-fuzz"]);
    assert_eq!(results.len(), 3, "expected every test to be reported: {results:?}");
    for (name, status) in &results {
        assert_eq!(status, "ignored", "{name} should be reported as skipped");
    }
}
