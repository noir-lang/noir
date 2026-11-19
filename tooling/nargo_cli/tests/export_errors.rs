//! Integration tests for how `nargo export` reports exported functions that fail to compile.

use std::process::Command;

use assert_cmd::prelude::*;
use assert_fs::TempDir;
use assert_fs::prelude::{FileWriteStr, PathAssert, PathChild};
use predicates::prelude::*;

/// Two exported functions that type-check but fail to compile, declared out of alphabetical
/// order, and one that compiles.
const SOURCE: &str = r#"
#[export]
fn zeta(x: Field) -> Field {
    std::static_assert(x == x + 1, "zeta fails");
    x
}

#[export]
fn alpha(x: Field) -> Field {
    std::static_assert(x == x + 2, "alpha fails");
    x
}

#[export]
fn compiles(x: Field) -> Field {
    x + 1
}
"#;

#[test]
fn export_reports_every_failing_function_and_writes_nothing() {
    let test_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&test_dir).arg("new").arg("--lib").arg("exports");
    cmd.assert().success();

    let project_dir = test_dir.child("exports");
    project_dir.child("src").child("lib.nr").write_str(SOURCE).unwrap();

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&project_dir).arg("export");
    let output = cmd.assert().failure().get_output().stderr.clone();
    let stderr = String::from_utf8(output).unwrap();

    // Both failures are reported, in the order `get_all_exported_functions_in_crate` returns the
    // functions: by name within a module.
    let alpha = stderr.find("alpha fails").expect("alpha's error should be reported");
    let zeta = stderr.find("zeta fails").expect("zeta's error should be reported");
    assert!(alpha < zeta, "alpha's error should come first:\n{stderr}");
    assert!(stderr.contains("Aborting due to 2 previous errors"), "{stderr}");

    project_dir.child("export").assert(predicate::path::missing());
}

#[test]
fn export_prints_each_functions_compilation_steps_in_one_piece() {
    let test_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&test_dir).arg("new").arg("--lib").arg("printed_exports");
    cmd.assert().success();

    let project_dir = test_dir.child("printed_exports");
    let source = (0..8)
        .map(|i| format!("#[export]\nfn f{i}(x: Field) -> Field {{\n    x * x + {i}\n}}\n"))
        .collect::<String>();
    project_dir.child("src").child("lib.nr").write_str(&source).unwrap();

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&project_dir).arg("export").arg("--show-ssa");
    let output = cmd.assert().success().get_output().stdout.clone();
    let stdout = String::from_utf8(output).unwrap();

    // Each printed pass starts with the function it belongs to. With the functions compiled one at
    // a time, all of one function's passes come before the next function's.
    let functions: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix("acir(inline) fn "))
        .filter_map(|rest| rest.split_whitespace().next())
        .collect();
    let mut seen = Vec::new();
    for function in &functions {
        if seen.last() != Some(function) {
            assert!(!seen.contains(function), "{function}'s passes are interleaved:\n{stdout}");
            seen.push(function);
        }
    }
    assert_eq!(seen.len(), 8, "expected every function's passes: {seen:?}");
}
