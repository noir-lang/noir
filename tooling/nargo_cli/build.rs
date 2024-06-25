use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

const GIT_COMMIT: &&str = &"GIT_COMMIT";

fn main() {
    // Only use build_data if the environment variable isn't set.
    if env::var(GIT_COMMIT).is_err() {
        build_data::set_GIT_COMMIT();
        build_data::set_GIT_DIRTY();
        build_data::no_debug_rebuilds();
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("execute.rs");
    let mut test_file = File::create(destination).unwrap();

    // Try to find the directory that Cargo sets when it is running; otherwise fallback to assuming the CWD
    // is the root of the repository and append the crate path
    let root_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir).parent().unwrap().parent().unwrap().to_path_buf(),
        Err(_) => env::current_dir().unwrap(),
    };
    let test_dir = root_dir.join("test_programs");

    // Rebuild if the tests have changed
    println!("cargo:rerun-if-changed=tests");
    // TODO: Running the tests changes the timestamps on test_programs files (file lock?).
    // That has the knock-on effect of then needing to rebuild the tests after running the tests.
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
    generate_trace_tests(&mut test_file, &test_dir);
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

/// Tests which aren't expected to work with the default minimum inliner cases.
const INLINER_MIN_OVERRIDES: [(&str, i64); 1] = [
    // 0 works if PoseidonHasher::write is tagged as `inline_always`, otherwise 22.
    ("eddsa", 0),
];

/// Tests which aren't expected to work with the default maximum inliner cases.
const INLINER_MAX_OVERRIDES: [(&str, i64); 0] = [];

/// These tests should only be run on exactly 1 inliner setting (the one given here)
const INLINER_OVERRIDES: [(&str, i64); 3] = [
    ("reference_counts_inliner_0", 0),
    ("reference_counts_inliner_min", i64::MIN),
    ("reference_counts_inliner_max", i64::MAX),
];

/// Some tests are expected to have warnings
/// These should be fixed and removed from this list.
const TESTS_WITH_EXPECTED_WARNINGS: [&str; 4] = [
    // TODO(https://github.com/noir-lang/noir/issues/6238): remove from list once issue is closed
    "brillig_cast",
    // TODO(https://github.com/noir-lang/noir/issues/6238): remove from list once issue is closed
    "macros_in_comptime",
    // We issue a "experimental feature" warning for all enums until they're stabilized
    "enums",
    "comptime_enums",
];

/// Tests for which we don't check that stdout matches the expected output.
const TESTS_WITHOUT_STDOUT_CHECK: [&str; 0] = [];

fn read_test_cases(
    test_data_dir: &Path,
    test_sub_dir: &str,
) -> impl Iterator<Item = (String, PathBuf)> {
    let test_data_dir = test_data_dir.join(test_sub_dir);
    let test_case_dirs =
        fs::read_dir(test_data_dir).unwrap().flatten().filter(|c| c.path().is_dir());

    test_case_dirs.into_iter().filter_map(|dir| {
        // When switching git branches we might end up with non-empty directories that have a `target`
        // directory inside them but no `Nargo.toml`.
        // These "tests" would always fail, but it's okay to ignore them so we do that here.
        if !dir.path().join("Nargo.toml").exists() {
            return None;
        }

        let test_name =
            dir.file_name().into_string().expect("Directory can't be converted to string");
        if test_name.contains('-') {
            panic!(
                "Invalid test directory: {test_name}. Cannot include `-`, please convert to `_`"
            );
        }
        Some((test_name, dir.path()))
    })
}

#[derive(Default)]
struct MatrixConfig {
    // Only used with execution, and only on selected tests.
    vary_brillig: bool,
    // Only seems to have an effect on the `execute_success` cases.
    vary_inliner: bool,
    // If there is a non-default minimum inliner aggressiveness to use with the brillig tests.
    min_inliner: i64,
    // If there is a non-default maximum inliner aggressiveness to use with the brillig tests.
    max_inliner: i64,
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
        if !cases.iter().any(|c| c.value() == matrix_config.max_inliner) {
            cases.push(Inliner::Custom(matrix_config.max_inliner));
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
            let inliner_range = matrix_config.min_inliner..=matrix_config.max_inliner;
            if *brillig && !inliner_range.contains(&inliner.value()) {
                continue;
            }
            test_cases.push(format!(
                "#[test_case::test_case(ForceBrillig({brillig}), Inliner({}))]",
                inliner.label()
            ));
        }
    }
    let test_cases = test_cases.join("\n");

    write!(
        test_file,
        r#"
{test_cases}
fn test_{test_name}(force_brillig: ForceBrillig, inliner_aggressiveness: Inliner) {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut nargo = Command::cargo_bin("nargo").unwrap();
    nargo.arg("--program-dir").arg(test_program_dir.clone());
    nargo.arg("{test_command}").arg("--force");
    nargo.arg("--inliner-aggressiveness").arg(inliner_aggressiveness.0.to_string());
    // Check whether the test case is non-deterministic
    nargo.arg("--check-non-determinism");
    // Allow more bytecode in exchange to catch illegal states.
    nargo.arg("--enable-brillig-debug-assertions");

    // Enable enums as an unstable feature
    nargo.arg("-Zenums");

    if force_brillig.0 {{
        nargo.arg("--force-brillig");

        // Set the maximum increase so that part of the optimization is exercised (it might fail).
        nargo.arg("--max-bytecode-increase-percent");
        nargo.arg("50");
    }}

    {test_content}
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

        fn remove_noise_lines(string: String) -> String {{
            string.lines().filter(|line| 
                !line.contains(\"Witness saved to\") && 
                    !line.contains(\"Circuit witness successfully solved\") &&
                    !line.contains(\"Waiting for lock\")
            ).collect::<Vec<&str>>().join(\"\n\")
        }}
    "
    )
    .unwrap();
    for (test_name, test_dir) in test_cases {
        let test_dir = test_dir.display();

        let test_content = if TESTS_WITHOUT_STDOUT_CHECK.contains(&test_name.as_str()) {
            "nargo.assert().success();"
        } else {
            r#"
            nargo.assert().success();

            let output = nargo.output().unwrap();
            let stdout = String::from_utf8(output.stdout).unwrap();
            let stdout = remove_noise_lines(stdout);

            let stdout_path = test_program_dir.join("stdout.txt");
            let expected_stdout = if stdout_path.exists() {
                String::from_utf8(fs::read(stdout_path).unwrap()).unwrap()
            } else {
                String::new()
            };

            // Remove any trailing newlines added by some editors
            let stdout = stdout.trim();
            let expected_stdout = expected_stdout.trim();

            if stdout != expected_stdout {
                println!("stdout does not match expected output. Expected:\n{expected_stdout}\n\nActual:\n{stdout}");
                assert_eq!(stdout, expected_stdout);
            }
            "#
        };

        generate_test_cases(
            test_file,
            &test_name,
            &test_dir,
            "execute",
            test_content,
            &MatrixConfig {
                vary_brillig: !IGNORED_BRILLIG_TESTS.contains(&test_name.as_str()),
                vary_inliner: true,
                min_inliner: min_inliner(&test_name),
                max_inliner: max_inliner(&test_name),
            },
        );
    }
    writeln!(test_file, "}}").unwrap();
}

fn max_inliner(test_name: &str) -> i64 {
    INLINER_MAX_OVERRIDES
        .iter()
        .chain(&INLINER_OVERRIDES)
        .find(|(n, _)| *n == test_name)
        .map(|(_, i)| *i)
        .unwrap_or(i64::MAX)
}

fn min_inliner(test_name: &str) -> i64 {
    INLINER_MIN_OVERRIDES
        .iter()
        .chain(&INLINER_OVERRIDES)
        .find(|(n, _)| *n == test_name)
        .map(|(_, i)| *i)
        .unwrap_or(i64::MIN)
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
        assert_eq!(num_opcodes.as_u64().expect("number of opcodes should fit in a u64"), 0, "expected the number of opcodes to be 0");
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

fn generate_trace_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_sub_dir = "trace";
    let test_data_dir = test_data_dir.join(test_sub_dir);

    let test_case_dirs =
        fs::read_dir(test_data_dir).unwrap().flatten().filter(|c| c.path().is_dir());

    let expected_messages = HashMap::from([("1_mul", vec!["Total tracing steps: 7"])]);

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
fn trace_{test_name}() {{
    use tempfile::tempdir;

    let test_program_dir = PathBuf::from("{test_dir}");

    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir.to_str().unwrap());
    cmd.arg("trace").arg("--trace-dir").arg(temp_dir.path());


    let trace_file_path = temp_dir.path().join("trace.json");
    let file_written_message = format!("Saved trace to {{:?}}", trace_file_path);

    cmd.assert().success().stdout(predicate::str::contains(file_written_message));"#,
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
    cmd.assert().success().stdout(predicate::str::contains("{message}"));"#
                    )
                    .expect("Could not write templated test file.");
                }
            }
            None => {}
        }

        write!(
            test_file,
            r#"

    let expected_trace_path = test_program_dir.join("expected_trace.json");
    let expected_trace = fs::read_to_string(expected_trace_path).unwrap();
    let expected_json: Value = serde_json::from_str(&expected_trace).unwrap();

    let actual_trace = fs::read_to_string(trace_file_path).unwrap();
    let actual_json: Value = serde_json::from_str(&actual_trace).unwrap();

    assert_eq!(expected_json, actual_json);
}}
"#
        )
        .expect("Could not write templated test file.");
    }
}
