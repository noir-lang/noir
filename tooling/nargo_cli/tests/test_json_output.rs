//! Integration tests for the suite events of `nargo test --format json` in a workspace.
//!
//! JSON output is live: tests from different packages interleave in the order they finish, and a
//! package's suite-finished event follows its last test. Each suite event names its package so a
//! reader can tell which suite finished.

use std::collections::BTreeMap;
use std::process::Command;

use assert_cmd::prelude::*;
use assert_fs::TempDir;
use assert_fs::prelude::{FileWriteStr, PathChild};
use serde_json::Value;

const ALPHA: &str = r#"
#[test]
fn a_one() {}

#[test]
fn a_two() {}

#[test]
fn a_fails() {
    assert(1 == 2);
}
"#;

const BETA: &str = r#"
#[test]
fn b_one() {}

#[test]
fn b_two() {}
"#;

/// `gamma` has no tests, so its suite starts and finishes without a test event in between.
const GAMMA: &str = "pub fn not_a_test() {}\n";

/// Runs `nargo test --format json` on a workspace of `alpha`, `beta` and `gamma` and returns the
/// emitted JSON lines in order.
fn json_events() -> Vec<Value> {
    let test_dir = TempDir::new().unwrap();
    test_dir
        .child("Nargo.toml")
        .write_str("[workspace]\nmembers = [\"alpha\", \"beta\", \"gamma\"]\n")
        .unwrap();
    for (name, source) in [("alpha", ALPHA), ("beta", BETA), ("gamma", GAMMA)] {
        let package = test_dir.child(name);
        package
            .child("Nargo.toml")
            .write_str(&format!(
                "[package]\nname = \"{name}\"\ntype = \"lib\"\nauthors = [\"\"]\n\n[dependencies]\n"
            ))
            .unwrap();
        package.child("src").child("lib.nr").write_str(source).unwrap();
    }

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&test_dir).args(["test", "--format", "json", "--test-threads", "2"]);
    let output = cmd.output().unwrap();
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).expect("every line of output is JSON"))
        .collect()
}

fn is_suite_end(event: &Value) -> bool {
    event["type"] == "suite" && event["event"] != "started"
}

#[test]
fn each_package_finishes_once_with_its_name_and_counts() {
    let events = json_events();

    let mut finished: BTreeMap<String, &Value> = BTreeMap::new();
    for event in events.iter().filter(|event| is_suite_end(event)) {
        let name = event["name"].as_str().expect("suite-finished event names its package");
        assert!(finished.insert(name.to_string(), event).is_none(), "{name} finished twice");
    }

    assert_eq!(finished.keys().collect::<Vec<_>>(), ["alpha", "beta", "gamma"]);

    let counts = |name: &str| {
        let event = finished[name];
        (
            event["event"].as_str().unwrap(),
            event["passed"].as_u64().unwrap(),
            event["failed"].as_u64().unwrap(),
        )
    };
    assert_eq!(counts("alpha"), ("failed", 2, 1));
    assert_eq!(counts("beta"), ("ok", 2, 0));
    assert_eq!(counts("gamma"), ("ok", 0, 0));
}

#[test]
fn a_package_finishes_after_its_last_test() {
    let events = json_events();

    for package in ["alpha", "beta", "gamma"] {
        let started = events
            .iter()
            .position(|event| {
                event["type"] == "suite" && event["event"] == "started" && event["name"] == package
            })
            .unwrap_or_else(|| panic!("{package} never started"));
        let finished = events
            .iter()
            .position(|event| is_suite_end(event) && event["name"] == package)
            .unwrap_or_else(|| panic!("{package} never finished"));
        let last_test_end = events.iter().rposition(|event| {
            event["type"] == "test" && event["event"] != "started" && event["suite"] == package
        });

        assert!(started < finished, "{package} finished before it started");
        if let Some(last_test_end) = last_test_end {
            assert!(last_test_end < finished, "{package} finished before its last test ended");
        }
    }
}
