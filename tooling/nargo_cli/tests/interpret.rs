//! Integration tests for `nargo interpret`.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

use assert_fs::prelude::{FileWriteStr, PathChild, PathCreateDir};

/// A `Prover.toml` whose array length disagrees with the declared parameter type
/// should surface a clean error, the same way `nargo execute` does, instead of
/// panicking inside the SSA input conversion.
#[test]
fn interpret_with_array_length_mismatch_reports_clean_error() {
    let project_dir = assert_fs::TempDir::new().unwrap();
    project_dir.child("src").create_dir_all().unwrap();
    project_dir
        .child("Nargo.toml")
        .write_str(
            r#"[package]
name = "interpret_length_mismatch"
type = "bin"
authors = []
compiler_version = ">=0.0.0"

[dependencies]
"#,
        )
        .unwrap();
    project_dir
        .child("src/main.nr")
        .write_str("fn main(a: pub [u32; 3]) -> pub u32 { a[0] }\n")
        .unwrap();
    project_dir.child("Prover.toml").write_str("a = [\"1\"]\n").unwrap();

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(project_dir.path()).arg("interpret");
    cmd.assert().failure().stderr(
        predicate::str::contains("does not match the specified type")
            .and(predicate::str::contains("array length != input length").not())
            .and(predicate::str::contains("application panicked").not()),
    );
}
