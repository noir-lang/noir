//! Integration tests for `nargo test` sharing one elaborated `Context` between the tests of a
//! package, across the worker threads.
//!
//! Sharing is only sound while compiling a test leaves nothing in the context that a later test can
//! see. `noirc_frontend::monomorphization::context_reuse_tests` asserts that at the level of the
//! monomorphizer, called directly against a shared `NodeInterner`.
//!
//! Asserted here is the property the runner actually depends on, through the runner itself: a
//! test's result must not depend on which tests compiled against the context before it or at the
//! same time. Each case compares a whole suite run against the same tests run one process at a
//! time.

use std::collections::BTreeMap;
use std::process::Command;

use assert_cmd::prelude::*;
use assert_fs::TempDir;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::{FileWriteStr, PathChild};

/// `a_transmute_mismatch` fails inside the generic `transmute_pair`, so the failure is raised part
/// way through monomorphizing it, at that call's instantiation. `#[test(should_fail)]` with no expected message reports the failure as a pass, which
/// is what makes this worth pinning through the runner: the suite is all green whatever the
/// context carries between tests, so a difference is only visible against the isolated runs.
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

/// A suite whose tests only differ in which impl of a trait they drive, so each test after the
/// first reaches a trait-method call site an earlier one has already resolved to a different impl.
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

/// Run `nargo test` in `project_dir` and return each test, as `<package>::<test>`, mapped to its
/// reported event (`ok`, `failed` or `ignored`).
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
        let (Some(suite), Some(name), Some(status)) =
            (event["suite"].as_str(), event["name"].as_str(), event["event"].as_str())
        else {
            continue;
        };
        let name = format!("{suite}::{name}");
        // Each test is announced when it starts and again when it finishes; only the outcome
        // is of interest here.
        if status == "started" {
            continue;
        }
        if let Some(previous) = results.insert(name.clone(), status.to_string()) {
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

#[test]
fn tests_sharing_a_context_across_threads_give_the_isolated_results() {
    let test_dir = TempDir::new().unwrap();
    let project_dir = new_project(&test_dir, "reuse_threads", COMPILE_FAILURE_BETWEEN_TESTS);

    let names = [
        "a_transmute_mismatch",
        "b_transmute_field",
        "c_transmute_u8",
        "d_transmute_mismatch_again",
        "e_transmute_field_again",
    ];

    // Several threads compile against the package's one context at the same time.
    let shared = test_results(&project_dir, &["--test-threads", "4"]);
    assert_eq!(shared, isolated_results(&project_dir, &names));
}

/// Creates a workspace of `members` packages, each with the source `source` gives for it.
fn new_workspace(
    test_dir: &TempDir,
    members: &[&str],
    source: impl Fn(&str) -> String,
) -> ChildPath {
    let workspace_dir = test_dir.child("workspace");
    std::fs::create_dir(&workspace_dir).unwrap();
    let members_list = members.iter().map(|member| format!("{member:?}")).collect::<Vec<_>>();
    workspace_dir
        .child("Nargo.toml")
        .write_str(&format!("[workspace]\nmembers = [{}]\n", members_list.join(", ")))
        .unwrap();
    for member in members {
        #[allow(deprecated)]
        let mut cmd = Command::cargo_bin("nargo").unwrap();
        cmd.current_dir(&workspace_dir).arg("new").arg(member);
        cmd.assert().success();
        workspace_dir
            .child(member)
            .child("src")
            .child("main.nr")
            .write_str(&source(member))
            .unwrap();
    }
    workspace_dir
}

#[test]
fn more_packages_than_threads_give_the_same_results_as_fewer() {
    let test_dir = TempDir::new().unwrap();
    let workspace_dir = new_workspace(&test_dir, &["p1", "p2", "p3"], |_| {
        COMPILE_FAILURE_BETWEEN_TESTS.to_string()
    });

    // With more packages than threads, the first test of each package elaborates it again
    // instead of keeping the context test collection built.
    let elaborated_per_package = test_results(&workspace_dir, &["--test-threads", "2"]);
    let kept_from_collection = test_results(&workspace_dir, &["--test-threads", "4"]);
    assert_eq!(elaborated_per_package, kept_from_collection);
    assert_eq!(elaborated_per_package.len(), 15, "{elaborated_per_package:?}");
}

/// A package whose elaboration prints `ELABORATED <package>`, so a run's output counts how many
/// times each package was elaborated. It has two regular tests and a fuzz test.
fn counts_its_elaborations(package: &str) -> String {
    format!(
        r#"
fn main() {{}}

#[allow(unused_variables)]
comptime global ELABORATED: Field = {{
    println("ELABORATED {package}");
    1
}};

#[test]
fn a() {{}}

#[test]
fn b() {{}}

#[test]
fn fuzzed(x: Field) {{
    assert_eq(x, x);
}}
"#
    )
}

/// How many times each package was elaborated by `nargo test` with `args` in `dir`.
fn elaborations(dir: &ChildPath, args: &[&str]) -> BTreeMap<String, usize> {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    // The fuzz test only needs to run, not to find anything.
    cmd.current_dir(dir).arg("test").args(["--fuzz-max-executions", "10"]).args(args);
    let output = cmd.output().expect("failed to run nargo test");
    assert!(output.status.success(), "nargo test failed: {output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();

    let mut counts = BTreeMap::new();
    for package in stdout.lines().filter_map(|line| line.strip_prefix("ELABORATED ")) {
        *counts.entry(package.to_string()).or_default() += 1;
    }
    counts
}

#[test]
fn tests_share_the_collection_elaboration_when_packages_fit_the_threads() {
    let test_dir = TempDir::new().unwrap();
    let workspace_dir = new_workspace(&test_dir, &["p1", "p2", "p3"], counts_its_elaborations);

    let once = |packages: &[&str]| -> BTreeMap<String, usize> {
        packages.iter().map(|package| (package.to_string(), 1)).collect()
    };
    let three_times = |packages: &[&str]| -> BTreeMap<String, usize> {
        packages.iter().map(|package| (package.to_string(), 3)).collect()
    };

    // Three packages on four threads: each package's tests use the context collecting them built.
    assert_eq!(elaborations(&workspace_dir, &["--test-threads", "4"]), once(&["p1", "p2", "p3"]));

    // Three packages on two threads: each package is elaborated again for its regular tests, which
    // share that elaboration, and once more for its fuzz test on the fuzz worker.
    assert_eq!(
        elaborations(&workspace_dir, &["--test-threads", "2"]),
        three_times(&["p1", "p2", "p3"])
    );

    // One selected package counts as one, however many members the workspace has.
    assert_eq!(
        elaborations(&workspace_dir, &["--package", "p1", "--test-threads", "1"]),
        once(&["p1"])
    );
}
