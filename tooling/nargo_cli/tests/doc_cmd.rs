//! Tests that `nargo doc` reports broken links in doc comments with the right exit status.

use assert_cmd::prelude::*;
use assert_fs::prelude::{FileWriteStr, PathChild};
use predicates::prelude::*;
use std::process::Command;

/// Creates a library crate named `name` with `source` as its `src/lib.nr`.
fn library_crate(name: &str, source: &str) -> assert_fs::TempDir {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("Nargo.toml")
        .write_str(&format!(
            "[package]\nname = \"{name}\"\ntype = \"lib\"\nauthors = [\"\"]\n\n[dependencies]\n"
        ))
        .unwrap();
    dir.child("src/lib.nr").write_str(source).unwrap();
    dir
}

/// Creates a library crate whose only doc comment links to an item that doesn't exist.
fn crate_with_broken_link() -> assert_fs::TempDir {
    library_crate("broken_link", "/// See [`NoSuchItem`].\npub fn foo() {}\n")
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

#[test]
fn link_in_impl_resolves_in_the_module_of_the_impl() {
    // `Square` is imported only in the module that declares the impl, not in the module of
    // `Wrapper` itself.
    let dir = library_crate(
        "impl_in_other_module",
        "
mod shapes {
    pub struct Square {}
}

mod wrapper {
    pub struct Wrapper {}
}

mod methods {
    use crate::shapes::Square;

    impl crate::wrapper::Wrapper {
        /// Returns a [`Square`].
        pub fn square(self) -> Square {
            Square {}
        }
    }
}
",
    );
    nargo_doc(&dir, &["--check"]).assert().success();
}
