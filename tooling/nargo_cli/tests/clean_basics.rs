use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

// Create a minimal, valid Noir project so workspace detection passes.
fn init_minimal_project(dir: &std::path::Path) {
    // Minimal manifest: name, type, version
    let mut f = fs::File::create(dir.join("Nargo.toml")).unwrap();
    writeln!(
        f,
        r#"[package]
name = "dummy"
type = "bin"
version = "0.1.0"
"#
    )
    .unwrap();

    // Optional but safe: ensure a source layout exists.
    let src = dir.join("src");
    fs::create_dir_all(&src).unwrap();
    // Minimal main; clean doesn't use it, but it keeps the package shape correct.
    let mut main = fs::File::create(src.join("main.nr")).unwrap();
    writeln!(main, "fn main() {{}}").unwrap();
}

#[test]
fn default_removes_target() {
    let tmp = TempDir::new().unwrap();
    init_minimal_project(tmp.path());
    fs::create_dir_all(tmp.path().join("target/foo")).unwrap();

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(tmp.path()).arg("clean");
    cmd.assert().success();

    assert!(!tmp.path().join("target").exists());
}

#[test]
fn clean_removes_local_crs_and_srs_only() {
    let tmp = TempDir::new().unwrap();
    init_minimal_project(tmp.path());
    fs::create_dir_all(tmp.path().join("target/crs")).unwrap();
    fs::create_dir_all(tmp.path().join("target/srs")).unwrap();
    fs::create_dir_all(tmp.path().join("target/other")).unwrap();

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(tmp.path()).args(["clean", "--crs"]);
    cmd.assert().success();

    assert!(tmp.path().join("target").exists());
    assert!(!tmp.path().join("target/crs").exists());
    assert!(!tmp.path().join("target/srs").exists());
    assert!(tmp.path().join("target/other").exists());
}

#[test]
fn dry_run_keeps_everything() {
    let tmp = TempDir::new().unwrap();
    init_minimal_project(tmp.path());
    fs::create_dir_all(tmp.path().join("target/crs")).unwrap();

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(tmp.path()).args(["clean", "--crs", "--dry-run"]);
    cmd.assert().success().stdout(predicate::str::contains("Dry run:"));

    assert!(tmp.path().join("target/crs").exists());
}
