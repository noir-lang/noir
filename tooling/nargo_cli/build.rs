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

    write!(
        test_file,
        r#"
{test_cases}
fn test_{test_name}(force_brillig: ForceBrillig, inliner_aggressiveness: Inliner) {{
    let test_program_dir = PathBuf::from("{test_dir}");

    let mut nargo = Command::cargo_bin("nargo").unwrap();
    nargo.arg("--program-dir").arg(test_program_dir);
    nargo.arg("{test_command}").arg("--force");
    nargo.arg("--inliner-aggressiveness").arg(inliner_aggressiveness.0.to_string());
    // Check whether the test case is non-deterministic
    nargo.arg("--check-non-determinism");

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
