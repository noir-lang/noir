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
    generate_compile_success_no_bug_tests(&mut test_file, &test_dir);
    generate_compile_failure_tests(&mut test_file, &test_dir);
    generate_noirc_frontend_failure_tests(&mut test_file, &test_dir);
    generate_plonky2_prove_success_tests(&mut test_file, &test_dir);
    generate_plonky2_prove_failure_tests(&mut test_file, &test_dir);
    generate_plonky2_prove_unsupported_tests(&mut test_file, &test_dir);
    generate_plonky2_prove_crash_tests(&mut test_file, &test_dir);
    generate_plonky2_verify_success_tests(&mut test_file, &test_dir);
    generate_plonky2_verify_failure_tests(&mut test_file, &test_dir);
    generate_plonky2_trace_tests(&mut test_file, &test_dir);
    generate_plonky2_show_plonky2_regression_tests(&mut test_file, &test_dir);
}

/// Some tests are explicitly ignored in brillig due to them failing.
/// These should be fixed and removed from this list.
const IGNORED_BRILLIG_TESTS: [&str; 11] = [
    // Takes a very long time to execute as large loops do not get simplified.
    "regression_4709",
    // bit sizes for bigint operation doesn't match up.
    "bigint",
    // ICE due to looking for function which doesn't exist.
    "fold_after_inlined_calls",
    "fold_basic",
    "fold_basic_nested_call",
    "fold_call_witness_condition",
    "fold_complex_outputs",
    "fold_distinct_return",
    "fold_fibonacci",
    "fold_numeric_generic_poseidon",
    // Expected to fail as test asserts on which runtime it is in.
    "is_unconstrained",
];

/// Discovers all test directories for the given `test_sub_dir` (if it exists) and returns an
/// iterator over pairs of the test name and the path to the directory that contains the test. If
/// there are no tests returns an empty iterator.
fn read_test_cases(
    test_data_dir: &Path,
    test_sub_dir: &str,
) -> Box<dyn Iterator<Item = (String, PathBuf)>> {
    let test_data_dir = test_data_dir.join(test_sub_dir);
    if let Ok(test_data_read_dir) = fs::read_dir(test_data_dir) {
        let test_case_dirs = test_data_read_dir
            .flatten()
            .filter(|c| c.path().is_dir() && c.path().join("Nargo.toml").exists());

        Box::new(test_case_dirs.into_iter().map(|dir| {
            let test_name =
                dir.file_name().into_string().expect("Directory can't be converted to string");
            if test_name.contains('-') {
                panic!(
                    "Invalid test directory: {test_name}. Cannot include `-`, please convert to `_`"
                );
            }
            (test_name, dir.path())
        }))
    } else {
        Box::new([].into_iter())
    }
}

fn generate_test_case(
    test_file: &mut File,
    test_type: &str,
    test_name: &str,
    test_dir: &std::path::Display,
    test_content: &str,
) {
    write!(
        test_file,
        r#"
#[test]
fn {test_type}_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut nargo = Command::cargo_bin("nargo").unwrap();
    nargo.arg("--program-dir").arg(test_program_dir);
    {test_content}
}}
"#
    )
    .expect("Could not write templated test file.");
}

fn generate_execution_success_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "execution_success";
    let test_cases = read_test_cases(test_data_dir, test_type);
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();

        generate_test_case(
            test_file,
            test_type,
            &test_name,
            &test_dir,
            r#"
                nargo.arg("execute").arg("--force");
            
                nargo.assert().success();"#,
        );

        if !IGNORED_BRILLIG_TESTS.contains(&test_name.as_str()) {
            generate_test_case(
                test_file,
                test_type,
                &format!("{test_name}_brillig"),
                &test_dir,
                r#"
                nargo.arg("execute").arg("--force").arg("--force-brillig");
            
                nargo.assert().success();"#,
            );
        }
    }
}

fn generate_execution_failure_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "execution_failure";
    let test_cases = read_test_cases(test_data_dir, test_type);
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();

        generate_test_case(
            test_file,
            test_type,
            &test_name,
            &test_dir,
            r#"
                nargo.arg("execute").arg("--force");
            
                nargo.assert().failure().stderr(predicate::str::contains("The application panicked (crashed).").not());"#,
        );
    }
}

fn generate_noir_test_success_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "noir_test_success";
    let test_cases = read_test_cases(test_data_dir, "noir_test_success");
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();

        generate_test_case(
            test_file,
            test_type,
            &test_name,
            &test_dir,
            r#"
        nargo.arg("test");
        
        nargo.assert().success();"#,
        );
    }
}

fn generate_noir_test_failure_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "noir_test_failure";
    let test_cases = read_test_cases(test_data_dir, test_type);
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();
        generate_test_case(
            test_file,
            test_type,
            &test_name,
            &test_dir,
            r#"
        nargo.arg("test");
        
        nargo.assert().failure();"#,
        );
    }
}

fn generate_compile_success_empty_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "compile_success_empty";
    let test_cases = read_test_cases(test_data_dir, test_type);
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();

        let assert_zero_opcodes = r#"
        let output = nargo.output().expect("Failed to execute command");

        if !output.status.success() {{
            panic!("`nargo info` failed with: {}", String::from_utf8(output.stderr).unwrap_or_default());
        }}
    
        // `compile_success_empty` tests should be able to compile down to an empty circuit.
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {{
            panic!("JSON was not well-formatted {:?}\n\n{:?}", e, std::str::from_utf8(&output.stdout))
        }});
        let num_opcodes = &json["programs"][0]["functions"][0]["opcodes"];
        assert_eq!(num_opcodes.as_u64().expect("number of opcodes should fit in a u64"), 0);
        "#;

        generate_test_case(
            test_file,
            test_type,
            &test_name,
            &test_dir,
            &format!(
                r#"
                nargo.arg("info").arg("--arithmetic-generics").arg("--json").arg("--force");
                
                {assert_zero_opcodes}"#,
            ),
        );
    }
}

fn generate_compile_success_contract_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "compile_success_contract";
    let test_cases = read_test_cases(test_data_dir, test_type);
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();

        generate_test_case(
            test_file,
            test_type,
            &test_name,
            &test_dir,
            r#"
        nargo.arg("compile").arg("--force");
        nargo.assert().success();"#,
        );
    }
}

/// Generate tests for checking that the contract compiles and there are no "bugs" in stderr
fn generate_compile_success_no_bug_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "compile_success_no_bug";
    let test_cases = read_test_cases(test_data_dir, test_type);
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();

        generate_test_case(
            test_file,
            test_type,
            &test_name,
            &test_dir,
            r#"
        nargo.arg("compile").arg("--force");
        nargo.assert().success().stderr(predicate::str::contains("bug:").not());"#,
        );
    }
}

fn generate_compile_failure_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "compile_failure";
    let test_cases = read_test_cases(test_data_dir, test_type);
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();

        generate_test_case(
            test_file,
            test_type,
            &test_name,
            &test_dir,
            r#"nargo.arg("compile").arg("--force");
        
        nargo.assert().failure().stderr(predicate::str::contains("The application panicked (crashed).").not());"#,
        );
    }
}

/// Test input programs that are expected to trigger an error in noirc_frontend.
fn generate_noirc_frontend_failure_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "noirc_frontend_failure";
    let test_cases = read_test_cases(test_data_dir, test_type);

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

    for (test_name, test_dir) in test_cases {
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
    let test_type = "plonky2_prove_success";
    let test_cases = read_test_cases(test_data_dir, test_type);
    for (test_name, test_dir) in test_cases {
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
    let test_type = "plonky2_prove_failure";
    let test_cases = read_test_cases(test_data_dir, test_type);

    let expected_messages = HashMap::from([("simple_add", vec!["Cannot satisfy constraint"])]);

    for (test_name, test_dir) in test_cases {
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
fn generate_plonky2_prove_unsupported_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "plonky2_prove_unsupported";
    let test_cases = read_test_cases(test_data_dir, test_type);
    for (test_name, test_dir) in test_cases {
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
fn generate_plonky2_prove_crash_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "plonky2_prove_crash";
    let test_cases = read_test_cases(test_data_dir, test_type);
    for (test_name, test_dir) in test_cases {
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
    let test_type = "plonky2_verify_success";
    let test_cases = read_test_cases(test_data_dir, test_type);
    for (test_name, test_dir) in test_cases {
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
    let test_type = "plonky2_verify_failure";
    let test_cases = read_test_cases(test_data_dir, test_type);

    let expected_messages = HashMap::from([
        ("zk_dungeon_verify_fail_1", vec!["Public inputs don't match proof"]),
        ("zk_dungeon_verify_fail_2", vec!["Expected argument `dagger.y`, but none was found"]),
    ]);

    for (test_name, test_dir) in test_cases {
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

fn generate_plonky2_trace_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "plonky2_trace";
    let test_cases = read_test_cases(test_data_dir, test_type);
    for (test_name, test_dir) in test_cases {
        write!(
            test_file,
            r#"
#[test]
fn plonky2_trace_{test_name}() {{
    use tempfile::tempdir;

    let test_program_dir_path = PathBuf::from("{test_dir}");

    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir_path.to_str().unwrap());
    cmd.arg("trace").arg("--trace-dir").arg(temp_dir.path());

    let trace_file_path = temp_dir.path().join("trace.json");
    let file_written_message = format!("Saved trace to {{:?}}", trace_file_path);

    cmd.assert().success().stdout(predicate::str::contains(file_written_message));

    let expected_trace_path = test_program_dir_path.join("expected_trace.json");
    let expected_trace = fs::read_to_string(expected_trace_path).expect("problem reading {{expected_trace_path}}");
    let mut expected_json: Value = serde_json::from_str(&expected_trace).unwrap();

    let actual_trace = fs::read_to_string(trace_file_path).expect("problem reading {{trace_file_path}}");
    let mut actual_json: Value = serde_json::from_str(&actual_trace).unwrap();

    // Ignore paths in test, because they need to be absolute and supporting them would make the
    // test too complicated.
    for trace_item in expected_json.as_array_mut().unwrap() {{
        if let Some(path) = trace_item.get_mut("Path") {{
            *path = json!("ignored-in-test");
        }}
    }}

    for trace_item in actual_json.as_array_mut().unwrap() {{
        if let Some(path) = trace_item.get_mut("Path") {{
            *path = json!("ignored-in-test");
        }}
    }}

    assert_eq!(expected_json, actual_json, "traces do not match");

    let expected_metadata_path = test_program_dir_path.join("expected_metadata.json");
    let expected_metadata = fs::read_to_string(expected_metadata_path).expect("problem reading expected_metadata.json");
    let mut expected_metadata_json: Value = serde_json::from_str(&expected_metadata).unwrap();
    if let Some(path) = expected_metadata_json.get_mut("workdir") {{
        *path = json!("ignored-in-test");
    }}

    let actual_metadata_path = temp_dir.path().join("trace_metadata.json");
    let actual_metadata = fs::read_to_string(actual_metadata_path).expect("problem reading trace_metadata.json");
    let mut actual_metadata_json: Value = serde_json::from_str(&actual_metadata).unwrap();
    if let Some(path) = actual_metadata_json.get_mut("workdir") {{
        *path = json!("ignored-in-test");
    }}

    assert_eq!(expected_metadata_json, actual_metadata_json, "trace metadata mismatch");

    let expected_paths_file_path = test_program_dir_path.join("expected_paths.json");
    let expected_paths = fs::read_to_string(expected_paths_file_path).expect("problem reading expected_paths.json");
    let expected_paths_json: Value = serde_json::from_str(&expected_paths).unwrap();
    let num_expected_paths = expected_paths_json.as_array().unwrap().len();

    let actual_paths_file_path = temp_dir.path().join("trace_paths.json");
    let actual_paths = fs::read_to_string(actual_paths_file_path).expect("problem reading actual_paths.json");
    let actual_paths_json: Value = serde_json::from_str(&actual_paths).unwrap();
    let num_actual_paths = actual_paths_json.as_array().unwrap().len();

    assert_eq!(num_expected_paths, num_actual_paths, "traces use a different number of files");
}}
"#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");
    }
}

/// Tests that compare the produced PLONKY2 backend output, against a given expected output.
fn generate_plonky2_show_plonky2_regression_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "plonky2_show_plonky2_regression";
    let test_cases = read_test_cases(test_data_dir, test_type);
    for (test_name, test_dir) in test_cases {
        write!(
            test_file,
            r#"
#[test]
fn plonky2_show_plonky2_regression_{test_name}() {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let plonky2_expected_output = format!("{{}}/plonky2.expected_output.txt", test_program_dir.display());
    let plonky2_generated_output = format!("{{}}/plonky2.generated_output.txt", test_program_dir.display());

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir);
    cmd.arg("prove");
    cmd.arg("--plonky2-print-file").arg(plonky2_generated_output.clone());

    cmd.assert().success();

    let mut cmd_diff = Command::new("diff");
    cmd_diff.arg("-c");
    cmd_diff.arg(plonky2_expected_output);
    cmd_diff.arg(plonky2_generated_output.clone());
    cmd_diff.assert().success();

    std::fs::remove_file(plonky2_generated_output).unwrap();
}}
            "#,
            test_dir = test_dir.display(),
        )
        .expect("Could not write templated test file.");
    }
}
