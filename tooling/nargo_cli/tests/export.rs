//! Integration tests for `nargo export`.

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

/// The functions whose SSA `stdout` prints, one entry per printed pass, in output order.
fn printed_functions(stdout: &str) -> Vec<&str> {
    stdout
        .lines()
        .filter(|line| line.starts_with("acir(") || line.starts_with("brillig("))
        .filter_map(|header| header.split(" fn ").nth(1))
        .filter_map(|rest| rest.split_whitespace().next())
        .collect()
}

/// Asserts that all of each function's passes are printed together, and that `expected`
/// functions are printed.
fn assert_printed_in_one_piece(stdout: &str, expected: usize) {
    let mut seen = Vec::new();
    for function in printed_functions(stdout) {
        if seen.last() != Some(&function) {
            assert!(!seen.contains(&function), "{function}'s passes are interleaved:\n{stdout}");
            seen.push(function);
        }
    }
    assert_eq!(seen.len(), expected, "expected every function's passes: {seen:?}");
}

/// A library with `count` exported functions named `<prefix>_<i>`.
fn exported_functions(prefix: &str, count: usize) -> String {
    (0..count)
        .map(|i| {
            format!(
                "#[export]\nfn {prefix}_{i}(x: Field) -> Field {{\n    let mut y = x;\n    for j in 0..32 {{\n        y = y * y + j as Field + {i};\n    }}\n    y\n}}\n"
            )
        })
        .collect()
}

#[test]
fn export_prints_each_functions_compilation_steps_in_one_piece() {
    let test_dir = TempDir::new().unwrap();
    let workspace_dir = test_dir.child("workspace");
    std::fs::create_dir(&workspace_dir).unwrap();
    workspace_dir.child("Nargo.toml").write_str("[workspace]\nmembers = [\"a\", \"b\"]\n").unwrap();
    for package in ["a", "b"] {
        let mut cmd = Command::cargo_bin("nargo").unwrap();
        cmd.current_dir(&workspace_dir).arg("new").arg("--lib").arg(package);
        cmd.assert().success();
        let source = exported_functions(package, 8);
        workspace_dir.child(package).child("src").child("lib.nr").write_str(&source).unwrap();
    }

    // Two packages with eight exports each: neither the packages nor their functions may compile
    // at the same time while their steps are printed.
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&workspace_dir).arg("export").arg("--show-ssa");
    let output = cmd.assert().success().get_output().stdout.clone();
    let stdout = String::from_utf8(output).unwrap();

    assert!(stdout.contains(" pure fn "), "expected passes after purity analysis:\n{stdout}");
    assert_printed_in_one_piece(&stdout, 16);
}
