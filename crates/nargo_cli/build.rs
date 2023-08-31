use rustc_version::{version, Version};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

fn check_rustc_version() {
    assert!(
        version().unwrap() >= Version::parse("1.66.0").unwrap(),
        "The minimal supported rustc version is 1.66.0."
    );
}

const GIT_COMMIT: &&str = &"GIT_COMMIT";

fn main() {
    // Rebuild if the tests have changed
    println!("cargo:rerun-if-changed=tests");

    check_rustc_version();

    // Only use build_data if the environment variable isn't set
    // The environment variable is always set when working via Nix
    if std::env::var(GIT_COMMIT).is_err() {
        build_data::set_GIT_COMMIT();
        build_data::set_GIT_DIRTY();
        build_data::no_debug_rebuilds();
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("execute.rs");
    let mut test_file = File::create(destination).unwrap();

    // Try to find the directory that Cargo sets when it is running; otherwise fallback to assuming the CWD
    // is the root of the repository and append the crate path
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => std::env::current_dir().unwrap().join("crates").join("nargo_cli"),
    };
    let test_dir = manifest_dir.join("tests");

    generate_execution_success_tests(&mut test_file, &test_dir);
    generate_compile_success_empty_tests(&mut test_file, &test_dir);
    generate_compile_success_contract_tests(&mut test_file, &test_dir);
    generate_compile_failure_tests(&mut test_file, &test_dir);
}

fn generate_execution_success_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_sub_dir = "execution_success";
    let test_data_dir = test_data_dir.join(test_sub_dir);

    let test_case_dirs =
        fs::read_dir(test_data_dir).unwrap().flatten().filter(|c| c.path().is_dir());

    for test_dir in test_case_dirs {
        let test_name =
            test_dir.file_name().into_string().expect("Directory can't be converted to string");
        if test_name.contains('-') {
            panic!(
                "Invalid test directory: {test_name}. Cannot include `-`, please convert to `_`"
            );
        };
        let test_dir = &test_dir.path();

        write!(
            test_file,
            r#"
#[test]
fn execution_success_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("execute");

    cmd.assert().success();
}}
            "#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");
    }
}

fn generate_compile_success_empty_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_sub_dir = "compile_success_empty";
    let test_data_dir = test_data_dir.join(test_sub_dir);

    let test_case_dirs =
        fs::read_dir(test_data_dir).unwrap().flatten().filter(|c| c.path().is_dir());

    for test_dir in test_case_dirs {
        let test_name =
            test_dir.file_name().into_string().expect("Directory can't be converted to string");
        if test_name.contains('-') {
            panic!(
                "Invalid test directory: {test_name}. Cannot include `-`, please convert to `_`"
            );
        };
        let test_dir = &test_dir.path();

        write!(
            test_file,
            r#"
#[test]
fn compile_success_empty_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("info");

    // `compile_success_empty` tests should be able to compile down to an empty circuit.
    cmd.assert().stdout(predicate::str::contains("| Package")
        .and(predicate::str::contains("| Language"))
        .and(predicate::str::contains("| ACIR Opcodes | Backend Circuit Size |"))
        .and(predicate::str::contains("| PLONKCSat {{ width: 3 }} |"))
        // This currently matches on there being zero acir opcodes due to the width of the cell.
        .and(predicate::str::contains("| 0            |")));
}}
            "#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");
    }
}

fn generate_compile_success_contract_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_sub_dir = "compile_success_contract";
    let test_data_dir = test_data_dir.join(test_sub_dir);

    let test_case_dirs =
        fs::read_dir(test_data_dir).unwrap().flatten().filter(|c| c.path().is_dir());

    for test_dir in test_case_dirs {
        let test_name =
            test_dir.file_name().into_string().expect("Directory can't be converted to string");
        if test_name.contains('-') {
            panic!(
                "Invalid test directory: {test_name}. Cannot include `-`, please convert to `_`"
            );
        };
        let test_dir = &test_dir.path();

        write!(
            test_file,
            r#"
#[test]
fn compile_success_contract_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("compile");

    cmd.assert().success();
}}
            "#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");
    }
}

fn generate_compile_failure_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_sub_dir = "compile_failure";
    let test_data_dir = test_data_dir.join(test_sub_dir);

    let test_case_dirs =
        fs::read_dir(test_data_dir).unwrap().flatten().filter(|c| c.path().is_dir());

    for test_dir in test_case_dirs {
        let test_name =
            test_dir.file_name().into_string().expect("Directory can't be converted to string");
        if test_name.contains('-') {
            panic!(
                "Invalid test directory: {test_name}. Cannot include `-`, please convert to `_`"
            );
        };
        let test_dir = &test_dir.path();

        write!(
            test_file,
            r#"
#[test]
fn compile_failure_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("execute");

    cmd.assert().failure();
}}
            "#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");
    }
}
