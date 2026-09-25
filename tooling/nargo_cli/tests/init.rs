use assert_cmd::prelude::*;
use assert_fs::prelude::{PathAssert, PathChild, PathCreateDir};
use predicates::prelude::*;
use std::process::Command;

#[test]
fn init_accepts_package_name_as_positional_argument() {
    let test_dir = assert_fs::TempDir::new().unwrap();
    let project_dir = test_dir.child("my-project");
    project_dir.create_dir_all().unwrap();

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.current_dir(&project_dir).arg("init").arg("my_project");
    cmd.assert().success();

    project_dir.child("src").assert(predicate::path::is_dir());
    project_dir.child("src/main.nr").assert(predicate::path::is_file());
    project_dir
        .child("Nargo.toml")
        .assert(predicate::path::is_file())
        .assert(predicate::str::contains(r#"name = "my_project""#));
}
