//! Integration tests for persisted compilation-cache eligibility.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

use assert_fs::prelude::{FileWriteStr, PathChild, PathCreateDir};

fn cache_project(source: &str) -> assert_fs::TempDir {
    let project_dir = assert_fs::TempDir::new().unwrap();
    project_dir.child("src").create_dir_all().unwrap();
    project_dir
        .child("Nargo.toml")
        .write_str(
            r#"[package]
name = "cache_validation"
type = "bin"
authors = []
compiler_version = ">=0.0.0"

[dependencies]
"#,
        )
        .unwrap();
    project_dir.child("src/main.nr").write_str(source).unwrap();

    project_dir
}

fn compile(project_dir: &assert_fs::TempDir, args: &[&str]) -> assert_cmd::assert::Assert {
    #[allow(deprecated)]
    let mut command = Command::cargo_bin("nargo").unwrap();
    command.arg("--program-dir").arg(project_dir.path()).arg("compile").args(args);
    command.assert()
}

#[test]
fn default_compile_rechecks_artifact_built_with_skipped_validation() {
    let project_dir = cache_project(
        r#"unconstrained fn plus_one(x: Field) -> Field {
    x + 1
}

fn main(x: Field) -> pub Field {
    // Safety: This deliberately exercises the missing-constraint diagnostic.
    unsafe { plus_one(x) }
}
"#,
    );

    compile(&project_dir, &["--skip-brillig-constraints-check", "--skip-underconstrained-check"])
        .success();

    compile(&project_dir, &[]).success().stderr(predicate::str::contains(
        "Brillig function call isn't properly covered by a manual constraint",
    ));
}

#[test]
fn compile_rechecks_cached_diagnostics_when_warning_policy_changes() {
    let project_dir = cache_project("fn main() -> pub Field { 1 }\n");
    let diagnostic = predicate::str::contains("Return variable contains a constant value");

    compile(&project_dir, &["--silence-warnings"]).success().stderr(diagnostic.clone().not());
    compile(&project_dir, &[]).success().stderr(diagnostic.clone());
    compile(&project_dir, &["--deny-warnings"]).failure().stderr(diagnostic);
}
