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

    test_case_dirs.into_iter().map(|dir| {
        let test_name =
            dir.file_name().into_string().expect("Directory can't be converted to string");
        if test_name.contains('-') {
            panic!(
                "Invalid test directory: {test_name}. Cannot include `-`, please convert to `_`"
            );
        }
        (test_name, dir.path())
    })
}

#[derive(Default)]
struct MatrixConfig {
    // Only used with execution, and only on selected tests.
    vary_brillig: bool,
    // Only seems to have an effect on the `execute_success` cases.
    vary_inliner: bool,
}

/// Generate all test cases for a given test directory.
/// These will be executed serially, but independently from other test directories.
/// Running multiple tests on the same directory concurrently risks overriding each
/// others compilation artifacts.
fn generate_test_cases(
    test_file: &mut File,
    test_name: &str,
    test_dir: &std::path::Display,
    test_command: &str,
    test_content: &str,
    matrix_config: &MatrixConfig,
) {
    let mutex_name = format! {"TEST_MUTEX_{}", test_name.to_uppercase()};
    let brillig_cases = if matrix_config.vary_brillig { "[false, true]" } else { "[false]" };
    let _inliner_cases = if matrix_config.vary_inliner { "[i64::MIN, 0, i64::MAX]" } else { "[0]" };
    // TODO (#6429): Remove this once the failing tests are fixed.
    let inliner_cases = "[i64::MAX]";
    write!(
        test_file,
        r#"
lazy_static::lazy_static! {{
    /// Prevent concurrent tests in the matrix from overwriting the compilation artifacts in {test_dir}
    static ref {mutex_name}: std::sync::Mutex<()> = std::sync::Mutex::new(());
}}

#[test_case::test_matrix(
    {brillig_cases}, 
    {inliner_cases}
)]
fn test_{test_name}(force_brillig: bool, inliner_aggressiveness: i64) {{
    // Ignore poisoning errors if some of the matrix cases failed.
    let _guard = {mutex_name}.lock().unwrap_or_else(|e| e.into_inner()); 

    let test_program_dir = PathBuf::from("{test_dir}");

    let mut nargo = Command::cargo_bin("nargo").unwrap();
    nargo.arg("--program-dir").arg(test_program_dir);
    nargo.arg("{test_command}").arg("--force");
    nargo.arg("--inliner-aggressiveness").arg(inliner_aggressiveness.to_string());
    if force_brillig {{
        nargo.arg("--force-brillig");
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
