use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

const GIT_COMMIT: &&str = &"GIT_COMMIT";

fn main() {
    // Only use build_data if the environment variable isn't set.
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
    let root_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir).parent().unwrap().parent().unwrap().to_path_buf(),
        Err(_) => std::env::current_dir().unwrap(),
    };
    let test_dir = root_dir.join("test_programs");

    // Rebuild if the tests have changed
    println!("cargo:rerun-if-changed=tests");
    println!("cargo:rerun-if-changed={}", test_dir.as_os_str().to_str().unwrap());

    generate_execution_success_tests(&mut test_file, &test_dir);
    generate_execution_failure_tests(&mut test_file, &test_dir);
    generate_noir_test_success_tests(&mut test_file, &test_dir);
    generate_noir_test_failure_tests(&mut test_file, &test_dir);
    generate_compile_success_empty_tests(&mut test_file, &test_dir);
    generate_compile_success_contract_tests(&mut test_file, &test_dir);
    generate_compile_failure_tests(&mut test_file, &test_dir);
    generate_noirc_frontend_failure_tests(&mut test_file, &test_dir);
    generate_plonky2_prove_success_tests(&mut test_file, &test_dir);
    generate_plonky2_prove_failure_tests(&mut test_file, &test_dir);
    // generate_plonky2_prove_unsupported_tests(&mut test_file, &test_dir);
    // generate_plonky2_prove_crash_tests(&mut test_file, &test_dir);
    generate_plonky2_verify_success_tests(&mut test_file, &test_dir);
    generate_plonky2_verify_failure_tests(&mut test_file, &test_dir);
}

/// Some tests are explicitly ignored in brillig due to them failing.
/// These should be fixed and removed from this list.
const IGNORED_BRILLIG_TESTS: [&str; 11] = [
    // Takes a very long time to execute as large loops do not get simplified.
    &"regression_4709",
    // bit sizes for bigint operation doesn't match up.
    &"bigint",
    // ICE due to looking for function which doesn't exist.
    &"fold_after_inlined_calls",
    &"fold_basic",
    &"fold_basic_nested_call",
    &"fold_call_witness_condition",
    &"fold_complex_outputs",
    &"fold_distinct_return",
    &"fold_fibonacci",
    &"fold_numeric_generic_poseidon",
    // Expected to fail as test asserts on which runtime it is in.
    &"is_unconstrained",
];

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

        let brillig_ignored =
            if IGNORED_BRILLIG_TESTS.contains(&test_name.as_str()) { "\n#[ignore]" } else { "" };

        write!(
            test_file,
            r#"
#[test]
fn execution_success_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("execute").arg("--force");

    cmd.assert().success();
}}

#[test]
fn execution_success_elaborator_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("execute").arg("--force").arg("--use-elaborator");

    cmd.assert().success();
}}

#[test]{brillig_ignored}
fn execution_success_{test_name}_brillig() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("execute").arg("--force").arg("--force-brillig");

    cmd.assert().success();
}}
            "#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");
    }
}

fn generate_execution_failure_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_sub_dir = "execution_failure";
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
fn execution_failure_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("execute").arg("--force");

    cmd.assert().failure().stderr(predicate::str::contains("The application panicked (crashed).").not());
}}

#[test]
fn execution_failure_elaborator_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("execute").arg("--force").arg("--use-elaborator");

    cmd.assert().failure().stderr(predicate::str::contains("The application panicked (crashed).").not());
}}
            "#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");
    }
}

fn generate_noir_test_success_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_sub_dir = "noir_test_success";
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
fn noir_test_success_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("test");

    cmd.assert().success();
}}

#[test]
fn noir_test_success_elaborator_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("test").arg("--use-elaborator");

    cmd.assert().success();
}}
            "#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");
    }
}

fn generate_noir_test_failure_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_sub_dir = "noir_test_failure";
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
fn noir_test_failure_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("test");

    cmd.assert().failure();
}}

#[test]
fn noir_test_failure_elaborator_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("test").arg("--use-elaborator");

    cmd.assert().failure();
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
    cmd.arg("--json");
    cmd.arg("--force");

    let output = cmd.output().expect("Failed to execute command");

    if !output.status.success() {{
        panic!("`nargo info` failed with: {{}}", String::from_utf8(output.stderr).unwrap_or_default());
    }}

    // `compile_success_empty` tests should be able to compile down to an empty circuit.
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {{
        panic!("JSON was not well-formatted {{:?}}\n\n{{:?}}", e, std::str::from_utf8(&output.stdout))
    }});
    let num_opcodes = &json["programs"][0]["functions"][0]["acir_opcodes"];
    assert_eq!(num_opcodes.as_u64().expect("number of opcodes should fit in a u64"), 0);
}}

#[test]
fn compile_success_empty_elaborator_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("info");
    cmd.arg("--json");
    cmd.arg("--force");
    cmd.arg("--use-elaborator");

    let output = cmd.output().expect("Failed to execute command");

    if !output.status.success() {{
        panic!("`nargo info` failed with: {{}}", String::from_utf8(output.stderr).unwrap_or_default());
    }}

    // `compile_success_empty` tests should be able to compile down to an empty circuit.
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {{
        panic!("JSON was not well-formatted {{:?}}\n\n{{:?}}", e, std::str::from_utf8(&output.stdout))
    }});
    let num_opcodes = &json["programs"][0]["functions"][0]["acir_opcodes"];
    assert_eq!(num_opcodes.as_u64().expect("number of opcodes should fit in a u64"), 0);
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
    cmd.arg("compile").arg("--force");

    cmd.assert().success();
}}
#[test]
fn compile_success_contract_elaborator_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("compile").arg("--force").arg("--use-elaborator");

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
    cmd.arg("compile").arg("--force");

    cmd.assert().failure().stderr(predicate::str::contains("The application panicked (crashed).").not());
}}
#[test]
fn compile_failure_elaborator_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("compile").arg("--force").arg("--use-elaborator");

    cmd.assert().failure().stderr(predicate::str::contains("The application panicked (crashed).").not());
}}
            "#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");
    }
}

/// Test input programs that are expected to trigger an error in noirc_frontend.
fn generate_noirc_frontend_failure_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_sub_dir = "noirc_frontend_failure";
    let test_data_dir = test_data_dir.join(test_sub_dir);

    let test_case_dirs =
        fs::read_dir(test_data_dir).unwrap().flatten().filter(|c| c.path().is_dir());

    let expected_messages = HashMap::from([
        (
            "invalid_bit_size",
            vec!["Use of invalid bit size 60", "Allowed bit sizes for integers are 1, 8, 32, 64"],
        ),
        ("diff_bit_sizes_add", vec!["Integers must have the same bit width"]),
        ("diff_bit_sizes_mul", vec!["Integers must have the same bit width"]),
        ("diff_bit_sizes_sub", vec!["Integers must have the same bit width"]),
        ("diff_bit_sizes_div", vec!["Integers must have the same bit width"]),
        (
            "field_plus_int",
            vec![
                "Types in a binary operation should match, but found Field and u64",
                "Unused expression result of type Field", // stanm: misleading warning
            ],
        ),
        ("field_comparison", vec!["Fields cannot be compared, try casting to an integer first"]),
    ]);

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
fn noirc_frontend_failure_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("prove");

    cmd.assert().failure().stderr(predicate::str::contains("The application panicked (crashed).").not());
            "#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");

        // Not all tests have expected messages, so match.
        match expected_messages.get(test_name.as_str()) {
            Some(messages) => {
                for message in messages.iter() {
                    write!(
                        test_file,
                        r#"
    cmd.assert().failure().stderr(predicate::str::contains("{message}"));"#
                    )
                    .expect("Could not write templated test file.");
                }
            }
            None => {}
        }

        write!(
            test_file,
            r#"
}}
"#
        )
        .expect("Could not write templated test file.");
    }
}

/// Tests using the experimental PLONKY2 backend as a proving engine that are expected to succeed.
fn generate_plonky2_prove_success_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_sub_dir = "plonky2_prove_success";
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
fn plonky2_prove_success_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("prove");

    cmd.assert().success();
}}
            "#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");
    }
}

/// Tests using the experimental PLONKY2 backend as a proving engine that are expected to fail.
fn generate_plonky2_prove_failure_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_sub_dir = "plonky2_prove_failure";
    let test_data_dir = test_data_dir.join(test_sub_dir);

    let test_case_dirs =
        fs::read_dir(test_data_dir).unwrap().flatten().filter(|c| c.path().is_dir());

    let expected_messages = HashMap::from([("simple_add", vec!["Cannot satisfy constraint"])]);

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
fn plonky2_prove_failure_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("prove");

    cmd.assert().failure().stderr(predicate::str::contains("The application panicked (crashed).").not());"#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");

        // Not all tests have expected messages, so match.
        match expected_messages.get(test_name.as_str()) {
            Some(messages) => {
                for message in messages.iter() {
                    write!(
                        test_file,
                        r#"
    cmd.assert().failure().stderr(predicate::str::contains("{message}"));"#
                    )
                    .expect("Could not write templated test file.");
                }
            }
            None => {}
        }

        write!(
            test_file,
            r#"
}}
"#
        )
        .expect("Could not write templated test file.");
    }
}

/// Tests using the experimental PLONKY2 backend as a proving engine that are expected to result in
/// an ICE with a message referring to unsupported features.
#[allow(dead_code)]
fn generate_plonky2_prove_unsupported_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_sub_dir = "plonky2_prove_unsupported";
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
fn plonky2_prove_unsupported_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("prove");

    cmd.assert().failure().stderr(predicate::str::contains("PLONKY2 backend does not support"));
    cmd.assert().failure().stderr(predicate::str::contains("The application panicked (crashed).").not());
}}
            "#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");
    }
}

/// Tests using the experimental PLONKY2 backend as a proving engine that are expected to crash.
/// TODO(stanm): Eliminate dead code before merging into master.
#[allow(dead_code)]
fn generate_plonky2_prove_crash_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_sub_dir = "plonky2_prove_crash";
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
fn plonky2_prove_crash_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("prove");

    cmd.assert().failure().stderr(predicate::str::contains("The application panicked (crashed)."));
}}
            "#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");
    }
}

/// Tests using the experimental PLONKY2 backend as a proving engine that are expected to succeed.
fn generate_plonky2_verify_success_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_sub_dir = "plonky2_verify_success";
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
fn plonky2_verify_success_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir.clone());
    cmd.arg("prove");

    cmd.assert().success();

    let mut cmd2 = Command::cargo_bin("nargo").unwrap();
    cmd2.arg("--program-dir").arg(test_program_dir);
    cmd2.arg("verify");

    cmd2.assert().success();
}}
            "#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");
    }
}

/// Tests using the experimental PLONKY2 backend as a proving engine that are expected to fail verification.
fn generate_plonky2_verify_failure_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_sub_dir = "plonky2_verify_failure";
    let test_data_dir = test_data_dir.join(test_sub_dir);

    let test_case_dirs =
        fs::read_dir(test_data_dir).unwrap().flatten().filter(|c| c.path().is_dir());

    let expected_messages = HashMap::from([
        ("zk_dungeon_verify_fail_1", vec!["Public inputs don't match proof"]),
        ("zk_dungeon_verify_fail_2", vec!["Expected argument `dagger.y`, but none was found"]),
    ]);

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
fn plonky2_verify_failure_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir.clone());
    cmd.arg("prove");

    cmd.assert().success();

    let mut cmd2 = Command::cargo_bin("nargo").unwrap();
    cmd2.arg("--program-dir").arg(test_program_dir);
    cmd2.arg("verify");
    cmd2.arg("--verifier-name").arg("VerifierTest");

    cmd2.assert().failure().stderr(predicate::str::contains("The application panicked (crashed).").not());"#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");

        // Not all tests have expected messages, so match.
        match expected_messages.get(test_name.as_str()) {
            Some(messages) => {
                for message in messages.iter() {
                    write!(
                        test_file,
                        r#"
    cmd2.assert().failure().stderr(predicate::str::contains("{message}"));"#
                    )
                    .expect("Could not write templated test file.");
                }
            }
            None => {}
        }

        write!(
            test_file,
            r#"
}}
"#
        )
        .expect("Could not write templated test file.");
    }
}
