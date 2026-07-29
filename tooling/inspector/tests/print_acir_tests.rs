use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::path::PathBuf;
use std::process::Command;

fn inspector_command() -> Command {
    #[allow(deprecated)]
    Command::cargo_bin("noir-inspector").unwrap()
}

// A different test program from the one `info_tests.rs` compiles, so the two
// test binaries never race on the same artifact file.
fn test_program_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test_programs/execution_success/a_3_add")
}

/// get test program artifact path, always recompiling to ensure correct version
fn test_artifact_path() -> PathBuf {
    let program_dir = test_program_dir();

    #[allow(deprecated)]
    let mut nargo = Command::cargo_bin("nargo").unwrap();
    nargo.arg("--program-dir").arg(&program_dir).arg("compile").arg("--force");

    nargo.assert().success();

    program_dir.join("target/a_3_add.json")
}

#[test]
fn test_print_acir_with_locations() {
    let artifact = test_artifact_path();

    inspector_command()
        .arg("print-acir")
        .arg(&artifact)
        .arg("--with-locations")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("// ")
                .and(predicate::str::contains("src/main.nr:4:5: assert(x == z)")),
        );
}

#[test]
fn test_print_acir_without_locations_has_no_annotations() {
    let artifact = test_artifact_path();

    inspector_command()
        .arg("print-acir")
        .arg(&artifact)
        .assert()
        .success()
        .stdout(predicate::str::contains("// ").not().and(predicate::str::contains("ASSERT")));
}
