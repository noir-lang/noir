//! Integration tests for `nargo fetch`, which resolves and downloads a package's
//! declared dependencies without compiling, checking, or testing.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

use assert_fs::prelude::{FileWriteStr, PathChild};

/// A freshly created project has no dependencies, so `nargo fetch` should succeed
/// without producing any output.
#[test]
fn fetch_with_no_dependencies() {
    let test_dir = assert_fs::TempDir::new().unwrap();

    let project_name = "no_deps";
    let project_dir = test_dir.child(project_name);

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&test_dir).arg("new").arg(project_name);
    cmd.assert().success();

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&project_dir).arg("fetch");
    cmd.assert().success().stdout(predicate::str::is_empty());
}

/// A local path dependency is used in place and requires no downloading, so `nargo fetch`
/// produces no output. The command must still succeed and be idempotent.
#[test]
fn fetch_with_local_dependency() {
    let test_dir = assert_fs::TempDir::new().unwrap();

    // A library package the binary will depend on.
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&test_dir).arg("new").arg("--lib").arg("my_lib");
    cmd.assert().success();

    // The binary package that depends on the library.
    let app_dir = test_dir.child("my_app");
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&test_dir).arg("new").arg("my_app");
    cmd.assert().success();

    app_dir
        .child("Nargo.toml")
        .write_str(
            "[package]\n\
             name = \"my_app\"\n\
             type = \"bin\"\n\
             authors = [\"\"]\n\n\
             [dependencies]\n\
             my_lib = { path = \"../my_lib\" }\n",
        )
        .unwrap();

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&app_dir).arg("fetch");
    cmd.assert().success().stdout(predicate::str::is_empty());

    // Running again is idempotent: it must still succeed.
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&app_dir).arg("fetch");
    cmd.assert().success().stdout(predicate::str::is_empty());
}
