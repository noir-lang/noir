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
    generate_compile_success_with_bug_tests(&mut test_file, &test_dir);
    generate_compile_failure_tests(&mut test_file, &test_dir);
    generate_noirc_frontend_failure_tests(&mut test_file, &test_dir);
    generate_plonky2_prove_success_tests(&mut test_file, &test_dir);
    generate_plonky2_prove_failure_tests(&mut test_file, &test_dir);
    generate_plonky2_prove_unsupported_tests(&mut test_file, &test_dir);
    generate_plonky2_prove_crash_tests(&mut test_file, &test_dir);
    generate_plonky2_verify_success_tests(&mut test_file, &test_dir);
    generate_plonky2_verify_failure_tests(&mut test_file, &test_dir);
    generate_plonky2_trace_tests(&mut test_file, &test_dir);
    generate_plonky2_trace_plonky2_tests(&mut test_file, &test_dir);
    generate_plonky2_show_plonky2_regression_tests(&mut test_file, &test_dir);
}

/// Some tests are explicitly ignored in brillig due to them failing.
/// These should be fixed and removed from this list.
const IGNORED_BRILLIG_TESTS: [&str; 10] = [
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

/// Tests which aren't expected to work with the default inliner cases.
const INLINER_MIN_OVERRIDES: [(&str, i64); 1] = [
    // 0 works if PoseidonHasher::write is tagged as `inline_always`, otherwise 22.
    ("eddsa", 0),
];

/// Some tests are expected to have warnings
/// These should be fixed and removed from this list.
const TESTS_WITH_EXPECTED_WARNINGS: [&str; 2] = [
    // TODO(https://github.com/noir-lang/noir/issues/6238): remove from list once issue is closed
    "brillig_cast",
    // TODO(https://github.com/noir-lang/noir/issues/6238): remove from list once issue is closed
    "macros_in_comptime",
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

#[derive(Default)]
struct MatrixConfig {
    // Only used with execution, and only on selected tests.
    vary_brillig: bool,
    // Only seems to have an effect on the `execute_success` cases.
    vary_inliner: bool,
    // If there is a non-default minimum inliner aggressiveness to use with the brillig tests.
    min_inliner: i64,
}

// Enum to be able to preserve readable test labels and also compare to numbers.
enum Inliner {
    Min,
    Default,
    Max,
    Custom(i64),
}

impl Inliner {
    fn value(&self) -> i64 {
        match self {
            Inliner::Min => i64::MIN,
            Inliner::Default => 0,
            Inliner::Max => i64::MAX,
            Inliner::Custom(i) => *i,
        }
    }
    fn label(&self) -> String {
        match self {
            Inliner::Min => "i64::MIN".to_string(),
            Inliner::Default => "0".to_string(),
            Inliner::Max => "i64::MAX".to_string(),
            Inliner::Custom(i) => i.to_string(),
        }
    }
}

/// Generate all test cases for a given test name (expected to be unique for the test directory),
/// based on the matrix configuration. These will be executed serially, but concurrently with
/// other test directories. Running multiple tests on the same directory would risk overriding
/// each others compilation artifacts, which is why this method injects a mutex shared by
/// all cases in the test matrix, as long as the test name and directory has a 1-to-1 relationship.
fn generate_test_cases(
    test_file: &mut File,
    test_name: &str,
    test_dir: &std::path::Display,
    test_command: &str,
    test_content: &str,
    matrix_config: &MatrixConfig,
) {
    let brillig_cases = if matrix_config.vary_brillig { vec![false, true] } else { vec![false] };
    let inliner_cases = if matrix_config.vary_inliner {
        let mut cases = vec![Inliner::Min, Inliner::Default, Inliner::Max];
        if !cases.iter().any(|c| c.value() == matrix_config.min_inliner) {
            cases.push(Inliner::Custom(matrix_config.min_inliner));
        }
        cases
    } else {
        vec![Inliner::Default]
    };

    // We can't use a `#[test_matrix(brillig_cases, inliner_cases)` if we only want to limit the
    // aggressiveness range for the brillig tests, and let them go full range on the ACIR case.
    let mut test_cases = Vec::new();
    for brillig in &brillig_cases {
        for inliner in &inliner_cases {
            if *brillig && inliner.value() < matrix_config.min_inliner {
                continue;
            }
            test_cases.push(format!(
                "#[test_case::test_case(ForceBrillig({brillig}), Inliner({}))]",
                inliner.label()
            ));
        }
    }
    let test_cases = test_cases.join("\n");

    // We need to isolate test cases in the same group, otherwise they overwrite each other's artifacts.
    // On CI we use `cargo nextest`, which runs tests in different processes; for this we use a file lock.
    // Locally we might be using `cargo test`, which run tests in the same process; in this case the file lock
    // wouldn't work, becuase the process itself has the lock, and it looks like it can have N instances without
    // any problems; for this reason we also use a `Mutex`.
    let mutex_name = format! {"TEST_MUTEX_{}", test_name.to_uppercase()};
    write!(
        test_file,
        r#"
lazy_static::lazy_static! {{
    /// Prevent concurrent tests in the matrix from overwriting the compilation artifacts in {test_dir}
    static ref {mutex_name}: std::sync::Mutex<()> = std::sync::Mutex::new(());
}}

{test_cases}
fn test_{test_name}(force_brillig: ForceBrillig, inliner_aggressiveness: Inliner) {{
    let test_program_dir = PathBuf::from("{test_dir}");

    // Ignore poisoning errors if some of the matrix cases failed.
    let mutex_guard = {mutex_name}.lock().unwrap_or_else(|e| e.into_inner());

    let file_guard = file_lock::FileLock::lock(
        test_program_dir.join("Nargo.toml"),
        true,
        file_lock::FileOptions::new().read(true).write(true).append(true)
    ).expect("failed to lock Nargo.toml");

    let mut nargo = Command::cargo_bin("nargo").unwrap();
    nargo.arg("--program-dir").arg(test_program_dir);
    nargo.arg("{test_command}").arg("--force");
    nargo.arg("--inliner-aggressiveness").arg(inliner_aggressiveness.0.to_string());

    if force_brillig.0 {{
        nargo.arg("--force-brillig");

        // Set the maximum increase so that part of the optimization is exercised (it might fail).
        nargo.arg("--max-bytecode-increase-percent");
        nargo.arg("50");

        // Check whether the test case is non-deterministic
        nargo.arg("--check-non-determinism");
    }}

    {test_content}

    drop(file_guard);
    drop(mutex_guard);
}}
"#
    )
    .expect("Could not write templated test file.");
}

fn generate_execution_success_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "execution_success";
    let test_cases = read_test_cases(test_data_dir, test_type);

    writeln!(
        test_file,
        "mod {test_type} {{
        use super::*;
    "
    )
    .unwrap();
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();

        generate_test_cases(
            test_file,
            &test_name,
            &test_dir,
            "execute",
            r#"
                nargo.assert().success();
            "#,
            &MatrixConfig {
                vary_brillig: !IGNORED_BRILLIG_TESTS.contains(&test_name.as_str()),
                vary_inliner: true,
                min_inliner: INLINER_MIN_OVERRIDES
                    .iter()
                    .find(|(n, _)| *n == test_name.as_str())
                    .map(|(_, i)| *i)
                    .unwrap_or(i64::MIN),
            },
        );
    }
    writeln!(test_file, "}}").unwrap();
}

fn generate_execution_failure_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "execution_failure";
    let test_cases = read_test_cases(test_data_dir, test_type);

    writeln!(
        test_file,
        "mod {test_type} {{
        use super::*;
    "
    )
    .unwrap();
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();

        generate_test_cases(
            test_file,
            &test_name,
            &test_dir,
            "execute",
            r#"
                nargo.assert().failure().stderr(predicate::str::contains("The application panicked (crashed).").not());
            "#,
            &MatrixConfig::default(),
        );
    }
    writeln!(test_file, "}}").unwrap();
}

fn generate_noir_test_success_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "noir_test_success";
    let test_cases = read_test_cases(test_data_dir, "noir_test_success");

    writeln!(
        test_file,
        "mod {test_type} {{
        use super::*;
    "
    )
    .unwrap();
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();

        generate_test_cases(
            test_file,
            &test_name,
            &test_dir,
            "test",
            r#"
                nargo.assert().success();
            "#,
            &MatrixConfig::default(),
        );
    }
    writeln!(test_file, "}}").unwrap();
}

fn generate_noir_test_failure_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "noir_test_failure";
    let test_cases = read_test_cases(test_data_dir, test_type);

    writeln!(
        test_file,
        "mod {test_type} {{
        use super::*;
    "
    )
    .unwrap();
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();
        generate_test_cases(
            test_file,
            &test_name,
            &test_dir,
            "test",
            r#"
                nargo.assert().failure();
            "#,
            &MatrixConfig::default(),
        );
    }
    writeln!(test_file, "}}").unwrap();
}

fn generate_compile_success_empty_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "compile_success_empty";
    let test_cases = read_test_cases(test_data_dir, test_type);

    writeln!(
        test_file,
        "mod {test_type} {{
        use super::*;
    "
    )
    .unwrap();
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();

        let mut assert_zero_opcodes = r#"
        let output = nargo.output().expect("Failed to execute command");

        if !output.status.success() {{
            panic!("`nargo info` failed with: {}", String::from_utf8(output.stderr).unwrap_or_default());
        }}
        "#.to_string();

        if !TESTS_WITH_EXPECTED_WARNINGS.contains(&test_name.as_str()) {
            assert_zero_opcodes += r#"
            nargo.assert().success().stderr(predicate::str::contains("warning:").not());
            "#;
        }

        assert_zero_opcodes += r#"
        // `compile_success_empty` tests should be able to compile down to an empty circuit.
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {{
            panic!("JSON was not well-formatted {:?}\n\n{:?}", e, std::str::from_utf8(&output.stdout))
        }});
        let num_opcodes = &json["programs"][0]["functions"][0]["opcodes"];
        assert_eq!(num_opcodes.as_u64().expect("number of opcodes should fit in a u64"), 0);
        "#;

        generate_test_cases(
            test_file,
            &test_name,
            &test_dir,
            "info",
            &format!(
                r#"
                nargo.arg("--json");
                {assert_zero_opcodes}
            "#,
            ),
            &MatrixConfig::default(),
        );
    }
    writeln!(test_file, "}}").unwrap();
}

fn generate_compile_success_contract_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "compile_success_contract";
    let test_cases = read_test_cases(test_data_dir, test_type);

    writeln!(
        test_file,
        "mod {test_type} {{
        use super::*;
    "
    )
    .unwrap();
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();

        generate_test_cases(
            test_file,
            &test_name,
            &test_dir,
            "compile",
            r#"
                nargo.assert().success().stderr(predicate::str::contains("warning:").not());
            "#,
            &MatrixConfig::default(),
        );
    }
    writeln!(test_file, "}}").unwrap();
}

/// Generate tests for checking that the contract compiles and there are no "bugs" in stderr
fn generate_compile_success_no_bug_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "compile_success_no_bug";
    let test_cases = read_test_cases(test_data_dir, test_type);

    writeln!(
        test_file,
        "mod {test_type} {{
        use super::*;
    "
    )
    .unwrap();
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();

        generate_test_cases(
            test_file,
            &test_name,
            &test_dir,
            "compile",
            r#"
                nargo.assert().success().stderr(predicate::str::contains("bug:").not());
            "#,
            &MatrixConfig::default(),
        );
    }
    writeln!(test_file, "}}").unwrap();
}

/// Generate tests for checking that the contract compiles and there are "bugs" in stderr
fn generate_compile_success_with_bug_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "compile_success_with_bug";
    let test_cases = read_test_cases(test_data_dir, test_type);

    writeln!(
        test_file,
        "mod {test_type} {{
        use super::*;
    "
    )
    .unwrap();
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();

        generate_test_cases(
            test_file,
            &test_name,
            &test_dir,
            "compile",
            r#"
                nargo.assert().success().stderr(predicate::str::contains("bug:"));
            "#,
            &MatrixConfig::default(),
        );
    }
    writeln!(test_file, "}}").unwrap();
}

fn generate_compile_failure_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "compile_failure";
    let test_cases = read_test_cases(test_data_dir, test_type);

    writeln!(
        test_file,
        "mod {test_type} {{
        use super::*;
    "
    )
    .unwrap();
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();

        generate_test_cases(
            test_file,
            &test_name,
            &test_dir,
            "compile",
            r#"
                nargo.assert().failure().stderr(predicate::str::contains("The application panicked (crashed).").not());
            "#,
            &MatrixConfig::default(),
        );
    }
    writeln!(test_file, "}}").unwrap();
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

fn generate_plonky2_trace_plonky2_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "plonky2_trace_plonky2";
    let test_cases = read_test_cases(test_data_dir, test_type);
    for (test_name, test_dir) in test_cases {
        write!(
            test_file,
            r#"
#[test]
fn plonky2_trace_plonky2_{test_name}() {{
    use tempfile::tempdir;

    let test_program_dir_path = PathBuf::from("{test_dir}");

    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir_path.to_str().unwrap());
    cmd.arg("trace").arg("--trace-dir").arg(temp_dir.path()).arg("--trace-plonky2");

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
    cmd.current_dir(test_program_dir);
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
