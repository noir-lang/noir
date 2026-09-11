//! Drives `nargo debug` through its REPL by feeding commands on stdin and asserting on the
//! locations, variables and results it prints.

use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::{FileWriteStr, PathChild};
use predicates::prelude::*;

const MAIN_NR: &str = r#"fn main(x: Field, y: pub Field) {
    let z = double(x);
    assert(z != y);
}

fn double(x: Field) -> Field {
    let two = 2;
    x * two
}

#[test]
fn test_simple_equal() {
    let x = 2;
    let y = 1 + 1;
    assert(x == y, "should be equal");
}
"#;

/// Writes a project containing `MAIN_NR` and returns its directory.
fn debug_project() -> TempDir {
    let project = TempDir::new().unwrap();
    project
        .child("Nargo.toml")
        .write_str(
            "[package]\nname = \"dbg_demo\"\ntype = \"bin\"\nauthors = [\"\"]\n\n[dependencies]\n",
        )
        .unwrap();
    project.child("src/main.nr").write_str(MAIN_NR).unwrap();
    project.child("Prover.toml").write_str("x = \"3\"\ny = \"2\"\n").unwrap();
    project
}

/// Runs `nargo debug <args>` in `project`, sending `commands` to the REPL.
fn nargo_debug(project: &TempDir, args: &[&str], commands: &str) -> assert_cmd::assert::Assert {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(project.path()).arg("debug").args(args).write_stdin(commands);
    cmd.assert()
}

#[test]
fn stops_at_first_statement_and_runs_to_completion() {
    let project = debug_project();

    nargo_debug(&project, &[], "continue\n")
        .success()
        .stdout(predicate::str::contains("At ").and(predicate::str::contains("main.nr:2 in main")))
        .stdout(predicate::str::contains("->    2 |     let z = double(x);"))
        .stdout(predicate::str::contains("Program completed. Return value: ()"));
}

#[test]
fn steps_into_over_and_out_showing_variables_and_call_stack() {
    let project = debug_project();

    let assert =
        nargo_debug(&project, &[], "vars\nstep\nstacktrace\nnext\nvars\nout\nvars\ncontinue\n")
            .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    let expected_in_order = [
        // `vars` at the entry point: only the arguments of `main`.
        "  x: Field = 0x03\n  y: Field = 0x02\n",
        // `step` follows the call into `double`.
        "main.nr:7 in double",
        // `stacktrace` from inside `double`.
        "#0  double at ",
        "#1  main at ",
        // `next` executes `let two = 2;` and stays in `double`.
        "main.nr:8 in double",
        "  two: Field = 0x02\n  x: Field = 0x03\n",
        // `out` returns to `main`, where the result is bound to `z`.
        "main.nr:3 in main",
        "  x: Field = 0x03\n  y: Field = 0x02\n  z: Field = 0x06\n",
        "Program completed. Return value: ()",
    ];

    let mut search_from = 0;
    for expected in expected_in_order {
        let found = stdout[search_from..].find(expected).unwrap_or_else(|| {
            panic!("expected {expected:?} after offset {search_from} in:\n{stdout}")
        });
        search_from += found + expected.len();
    }
}

#[test]
fn breakpoint_stops_continue() {
    let project = debug_project();

    nargo_debug(&project, &[], "break 3\nbreakpoints\ncontinue\ncontinue\n")
        .success()
        .stdout(
            predicate::str::contains("Breakpoint set at ")
                .and(predicate::str::contains("main.nr:3")),
        )
        .stdout(
            predicate::str::contains("Breakpoint hit.\nAt ")
                .and(predicate::str::contains("main.nr:3 in main")),
        )
        .stdout(predicate::str::contains("Program completed. Return value: ()"));
}

#[test]
fn reports_assertion_failure() {
    let project = debug_project();
    // `double(1) == 2 == y`, so the assertion in `main` fails.
    project.child("Prover.toml").write_str("x = \"1\"\ny = \"2\"\n").unwrap();

    nargo_debug(&project, &[], "continue\n")
        .success()
        .stderr(predicate::str::contains("Assertion failed"))
        .stdout(predicate::str::contains("Program completed").not());
}

#[test]
fn debugs_test_function_and_reports_result() {
    let project = debug_project();

    nargo_debug(&project, &["--test-name", "test_simple_equal"], "continue\n")
        .success()
        .stdout(predicate::str::contains("main.nr:13 in test_simple_equal"))
        .stdout(predicate::str::contains("[dbg_demo] Testing test_simple_equal ... ok"));
}

#[test]
fn restart_starts_a_fresh_session() {
    let project = debug_project();

    let assert = nargo_debug(&project, &[], "restart\ncontinue\n").success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(stdout.contains("Restarting debugger..."), "{stdout}");
    assert_eq!(stdout.matches("Debugger started.").count(), 2, "{stdout}");
    assert_eq!(stdout.matches("main.nr:2 in main").count(), 2, "{stdout}");
}
