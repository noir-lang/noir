//! Tests that `nargo doc` reports broken links in doc comments with the right exit status.

use assert_cmd::prelude::*;
use assert_fs::prelude::{FileWriteStr, PathChild};
use predicates::prelude::*;
use std::process::Command;

/// Creates a library crate whose only doc comment links to an item that doesn't exist.
fn crate_with_broken_link() -> assert_fs::TempDir {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("Nargo.toml")
        .write_str("[package]\nname = \"broken_link\"\ntype = \"lib\"\nauthors = [\"\"]\n\n[dependencies]\n")
        .unwrap();
    dir.child("src/lib.nr").write_str("/// See [`NoSuchItem`].\npub fn foo() {}\n").unwrap();
    dir
}

fn nargo_doc(dir: &assert_fs::TempDir, args: &[&str]) -> Command {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("doc").arg("--program-dir").arg(dir.path()).args(args);
    cmd
}

#[test]
fn broken_link_is_a_warning_by_default() {
    let dir = crate_with_broken_link();
    nargo_doc(&dir, &[])
        .assert()
        .success()
        .stderr(predicate::str::contains("warning: Unresolved link to `NoSuchItem`"));
}

#[test]
fn broken_link_fails_with_deny_warnings() {
    let dir = crate_with_broken_link();
    nargo_doc(&dir, &["--deny-warnings"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error: Unresolved link to `NoSuchItem`"));
}

#[test]
fn broken_link_fails_with_check() {
    let dir = crate_with_broken_link();
    nargo_doc(&dir, &["--check"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("doc comments contain 1 broken link"));
}
