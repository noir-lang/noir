use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

const GIT_COMMIT: &&str = &"GIT_COMMIT";

fn main() -> Result<(), String> {
    // Only use build_data if the environment variable isn't set.
    if env::var(GIT_COMMIT).is_err() {
        build_data::set_GIT_COMMIT()?;
        build_data::set_GIT_DIRTY()?;
        build_data::no_debug_rebuilds()?;
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
    generate_execution_panic_tests(&mut test_file, &test_dir);
    generate_noir_test_success_tests(&mut test_file, &test_dir);
    generate_noir_test_failure_tests(&mut test_file, &test_dir);
    generate_compile_success_empty_tests(&mut test_file, &test_dir);
    generate_compile_success_contract_tests(&mut test_file, &test_dir);
    generate_compile_success_no_bug_tests(&mut test_file, &test_dir);
    generate_compile_success_with_bug_tests(&mut test_file, &test_dir);
    generate_compile_failure_tests(&mut test_file, &test_dir);

    generate_minimal_execution_success_tests(&mut test_file, &test_dir);
    generate_interpret_execution_success_tests(&mut test_file, &test_dir);
    generate_interpret_execution_failure_tests(&mut test_file, &test_dir);

    generate_comptime_interpret_execution_success_tests(&mut test_file, &test_dir);
    generate_comptime_interpret_execution_failure_tests(&mut test_file, &test_dir);

    generate_fuzzing_failure_tests(&mut test_file, &test_dir);
    generate_trace_tests(&mut test_file, &test_dir);

    generate_nargo_expand_execution_success_tests(&mut test_file, &test_dir);
    generate_nargo_expand_compile_tests_with_ignore_list(
        "compile_success_empty",
        &mut test_file,
        &test_dir,
        &IGNORED_NARGO_EXPAND_COMPILE_SUCCESS_EMPTY_TESTS,
    );
    generate_nargo_expand_compile_tests("compile_success_contract", &mut test_file, &test_dir);
    generate_nargo_expand_compile_tests_with_ignore_list(
        "compile_success_no_bug",
        &mut test_file,
        &test_dir,
        &IGNORED_NARGO_EXPAND_COMPILE_SUCCESS_NO_BUG_TESTS,
    );
    generate_nargo_expand_compile_tests_with_ignore_list(
        "compile_success_with_bug",
        &mut test_file,
        &test_dir,
        &IGNORED_NARGO_EXPAND_COMPILE_SUCCESS_WITH_BUG_TESTS,
    );

    Ok(())
}

/// Some tests are explicitly ignored in brillig due to them failing.
/// These should be fixed and removed from this list.
const IGNORED_BRILLIG_TESTS: [&str; 11] = [
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
    // The output depends on function IDs of lambdas, and with --force-brillig we only get one kind.
    "regression_10158",
];

/// Tests which aren't expected to work with the default minimum inliner cases.
const INLINER_MIN_OVERRIDES: [(&str, i64); 1] = [
    // 0 works if PoseidonHasher::write is tagged as `inline_always`, otherwise 22.
    ("eddsa", 0),
];

/// Tests which aren't expected to work with the default maximum inliner cases.
const INLINER_MAX_OVERRIDES: [(&str, i64); 0] = [];

/// These tests should only be run on exactly 1 inliner setting (the one given here)
const INLINER_OVERRIDES: [(&str, i64); 4] = [
    ("reference_counts_inliner_0", 0),
    ("reference_counts_inliner_min", i64::MIN),
    ("reference_counts_inliner_max", i64::MAX),
    ("reference_counts_vectors_inliner_0", 0),
];

/// Some tests are expected to have warnings
/// These should be fixed and removed from this list.
const TESTS_WITH_EXPECTED_WARNINGS: [&str; 5] = [
    // TODO(https://github.com/noir-lang/noir/issues/6238): remove from list once issue is closed
    "brillig_cast",
    // TODO(https://github.com/noir-lang/noir/issues/6238): remove from list once issue is closed
    "macros_in_comptime",
    // We issue a "experimental feature" warning for all enums until they're stabilized
    "enums",
    "comptime_enums",
    // Testing unreachable instructions
    "brillig_continue_break",
];

/// `nargo interpret` ignored tests, either because they don't currently work or
/// because they are too slow to run.
const IGNORED_INTERPRET_EXECUTION_TESTS: [&str; 2] = [
    // slow
    "regression_4709",
    // Doesn't match Brillig, but the expected ref-count of 5 has comments which
    // suggest it's not exactly clear why we get that exact value anyway.
    "reference_counts_inliner_max",
];

/// `nargo execute --force-comptime` ignored tests because of bugs or because some
/// programs don't behave the same way in comptime (for example: reference counting).
const IGNORED_COMPTIME_INTERPRET_EXECUTION_TESTS: [&str; 5] = [
    // These check reference counts, which aren't tracked in comptime code
    "reference_counts_inliner_0",
    "reference_counts_inliner_max",
    "reference_counts_inliner_min",
    "reference_counts_vectors_inliner_0",
    // Enums are currently unsupported in comptime code
    "regression_7323",
];

const IGNORED_COMPTIME_INTERPRET_EXECUTION_FAILURE_TESTS: [&str; 0] = [];
/// We usually check that the stdout of `nargo execute --force-comptime` matches
/// that of `nargo execute`, but in some cases the output doesn't match and it's not clear
/// this can be solved.
/// There are two Noir types that show out differently in comptime: functions and references.
const IGNORED_COMPTIME_INTERPRET_EXECUTION_STDOUT_CHECK_TESTS: [&str; 4] =
    ["debug_logs", "regression_10156", "regression_10158", "regression_9578"];

/// `nargo execute --minimal-ssa` ignored tests
const IGNORED_MINIMAL_EXECUTION_TESTS: [&str; 16] = [
    // internal error: entered unreachable code: unsupported function call type Intrinsic(AssertConstant)
    // These tests contain calls to `assert_constant`, which are evaluated and removed in the full SSA
    // pipeline, but in the minimal they are untouched, and trying to remove them causes a failure because
    // we don't have the other passes that would turn expressions into constants.
    "array_to_vector_constant_length",
    "static_assert_empty_loop",
    "brillig_cow_regression",
    "brillig_pedersen",
    "import",
    "merkle_insert",
    "pedersen_check",
    "pedersen_hash",
    "pedersen_commitment",
    "simple_shield",
    "strings",
    // The minimal SSA pipeline only works with Brillig: \'zeroed_lambda\' needs to be unconstrained
    "lambda_from_dynamic_if",
    "regression_10156",
    // This relies on maximum inliner setting
    "reference_counts_inliner_max",
    "reference_counts_inliner_min",
    "reference_counts_inliner_0",
];

/// These tests are ignored because making them work involves a more complex test code that
/// might not be worth it.
/// Others are ignored because of existing bugs in `nargo expand`.
/// As the bugs are fixed these tests should be removed from this list.
const IGNORED_NARGO_EXPAND_EXECUTION_TESTS: [&str; 10] = [
    // There's nothing special about this program but making it work with a custom entry would involve
    // having to parse the Nargo.toml file, etc., which is not worth it
    "custom_entry",
    // There's no "src/main.nr" here so it's trickier to make this work
    "diamond_deps_0",
    // bug
    "numeric_type_alias",
    "negative_associated_constants",
    // There's no "src/main.nr" here so it's trickier to make this work
    "overlapping_dep_and_mod",
    // bug
    "regression_9116",
    // bug
    "regression_10466",
    // bug
    "trait_associated_constant",
    // There's no "src/main.nr" here so it's trickier to make this work
    "workspace",
    // There's no "src/main.nr" here so it's trickier to make this work
    "workspace_default_member",
];

/// Tests for which we don't check that stdout matches the expected output.
const TESTS_WITHOUT_STDOUT_CHECK: [&str; 0] = [];

/// These tests are ignored because of existing bugs in `nargo expand`.
/// As the bugs are fixed these tests should be removed from this list.
/// (some are ignored on purpose for the same reason as `IGNORED_NARGO_EXPAND_EXECUTION_TESTS`)
const IGNORED_NARGO_EXPAND_COMPILE_SUCCESS_EMPTY_TESTS: [&str; 9] = [
    // bug
    "associated_type_bounds",
    // bug
    "enums",
    // There's no "src/main.nr" here so it's trickier to make this work
    "overlapping_dep_and_mod",
    // this one works, but copying its `Nargo.toml` file to somewhere else doesn't work
    // because it references another project by a relative path
    "reexports",
    // bug
    "trait_function_calls",
    // bug
    "trait_method_mut_self",
    // bug
    "trait_static_methods",
    // There's no "src/main.nr" here so it's trickier to make this work
    "workspace_reexport_bug",
    // bug
    "trait_call_in_global",
];

/// These tests are ignored because of existing bugs in `nargo expand`.
/// As the bugs are fixed these tests should be removed from this list.
const IGNORED_NARGO_EXPAND_COMPILE_SUCCESS_NO_BUG_TESTS: [&str; 17] = [
    "noirc_frontend_tests_check_trait_as_type_as_fn_parameter",
    "noirc_frontend_tests_check_trait_as_type_as_two_fn_parameters",
    "noirc_frontend_tests_enums_match_on_empty_enum",
    "noirc_frontend_tests_traits_trait_alias_polymorphic_inheritance",
    "noirc_frontend_tests_traits_trait_alias_single_member",
    "noirc_frontend_tests_traits_trait_alias_two_members",
    "noirc_frontend_tests_traits_trait_impl_with_where_clause_with_trait_with_associated_numeric",
    "noirc_frontend_tests_traits_accesses_associated_type_inside_trait_impl_using_self",
    "noirc_frontend_tests_traits_accesses_associated_type_inside_trait_using_self",
    "noirc_frontend_tests_u32_globals_as_sizes_in_types",
    // This creates a struct at comptime which, expanded, gives a visibility error
    "noirc_frontend_tests_visibility_visibility_bug_inside_comptime",
    "noirc_frontend_tests_aliases_identity_numeric_type_alias_works",
    "noirc_frontend_tests_aliases_type_alias_to_numeric_as_generic",
    "noirc_frontend_tests_aliases_type_alias_to_numeric_generic",
    "noirc_frontend_tests_traits_trait_bound_on_implementing_type",
    "function_registry",
    "regression_10887", // expands into global struct with private fields
];

const IGNORED_NARGO_EXPAND_COMPILE_SUCCESS_WITH_BUG_TESTS: [&str; 0] = [];

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
/// based on the matrix configuration.
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

    #[allow(unused_mut)]
    let mut nargo = setup_nargo(&test_program_dir, "{test_command}", force_brillig, inliner_aggressiveness);
    {test_content}
}}
"#
    )
    .expect("Could not write templated test file.");
}

/// Generate fuzzing tests, where the noir program is fuzzed with one thread for 120 seconds.
/// We expect that a failure is found in that time
fn generate_fuzzing_test_case(
    test_file: &mut File,
    test_name: &str,
    test_dir: &std::path::Display,
    test_content: &str,
    timeout: usize,
) {
    let timeout_str = timeout.to_string();
    write!(
        test_file,
        r#"
#[test]
fn test_{test_name}() {{

    let corpus_dir = assert_fs::TempDir::new().unwrap();
    let fuzzing_failure_dir = assert_fs::TempDir::new().unwrap();
    let test_program_dir = PathBuf::from("{test_dir}");
    #[allow(deprecated)]
    let mut nargo = Command::cargo_bin("nargo").unwrap();
    nargo.arg("--program-dir").arg(test_program_dir);
    nargo.arg("fuzz").arg("--timeout").arg("{timeout_str}");
    nargo.arg("--corpus-dir").arg(corpus_dir.path());
    nargo.arg("--fuzzing-failure-dir").arg(fuzzing_failure_dir.path());

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

        let check_stdout = !TESTS_WITHOUT_STDOUT_CHECK.contains(&test_name.as_str());

        generate_test_cases(
            test_file,
            &test_name,
            &test_dir,
            "execute",
            &format!("execution_success(nargo, test_program_dir, {check_stdout});",),
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
            "execution_failure(nargo);",
            &MatrixConfig { vary_brillig: true, ..Default::default() },
        );
    }
    writeln!(test_file, "}}").unwrap();
}

fn generate_execution_panic_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "execution_panic";
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
            "execution_panic(nargo);",
            &MatrixConfig::default(),
        );
    }
    writeln!(test_file, "}}").unwrap();
}

fn generate_comptime_interpret_execution_success_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "execution_success";
    let test_cases = read_test_cases(test_data_dir, test_type);

    writeln!(
        test_file,
        "mod comptime_interpret_{test_type} {{
        use super::*;
    "
    )
    .unwrap();
    for (test_name, test_dir) in test_cases {
        let should_panic =
            if IGNORED_COMPTIME_INTERPRET_EXECUTION_TESTS.contains(&test_name.as_str()) {
                "#[should_panic]"
            } else {
                ""
            };
        let check_stdout =
            !IGNORED_COMPTIME_INTERPRET_EXECUTION_STDOUT_CHECK_TESTS.contains(&test_name.as_str());

        let test_dir = test_dir.display();

        write!(
            test_file,
            r#"
            #[test]
            {should_panic}
            fn test_{test_name}() {{
                let test_program_dir = PathBuf::from("{test_dir}");
                nargo_execute_comptime(test_program_dir, {check_stdout});
            }}
            "#
        )
        .unwrap();
    }
    writeln!(test_file, "}}").unwrap();
}

fn generate_comptime_interpret_execution_failure_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "execution_failure";
    let test_cases = read_test_cases(test_data_dir, test_type);

    writeln!(
        test_file,
        "mod comptime_interpret_{test_type} {{
          use super::*;
      "
    )
    .unwrap();

    for (test_name, test_dir) in test_cases {
        let should_panic =
            if IGNORED_COMPTIME_INTERPRET_EXECUTION_FAILURE_TESTS.contains(&test_name.as_str()) {
                "#[should_panic]"
            } else {
                ""
            };

        let test_dir = test_dir.display();

        write!(
            test_file,
            r#"
              #[test]
              {should_panic}
              fn test_{test_name}() {{
                  let test_program_dir = PathBuf::from("{test_dir}");
                  nargo_execute_comptime_expect_failure(test_program_dir);
              }}
              "#
        )
        .unwrap();
    }
    writeln!(test_file, "}}").unwrap();
}

/// Generate tests for fuzzing which find failures in the fuzzed program.
fn generate_fuzzing_failure_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "fuzzing_failure";
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

        generate_fuzzing_test_case(
            test_file,
            &test_name,
            &test_dir,
            r#"
                nargo.assert().failure().stderr(
                    predicate::str::contains("Failing input").and(
                    predicate::str::contains("got a different failing assertion").not())
                );
            "#,
            240,
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
            "noir_test_success(nargo);",
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
            "noir_test_failure(nargo);",
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

        generate_test_cases(
            test_file,
            &test_name,
            &test_dir,
            "info",
            &format!(
                "compile_success_empty(nargo, test_program_dir, {}, force_brillig, inliner_aggressiveness);",
                !TESTS_WITH_EXPECTED_WARNINGS.contains(&test_name.as_str())
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
            "compile_success_contract(nargo, test_program_dir, force_brillig, inliner_aggressiveness);",
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
            "compile_success_no_bug(nargo);",
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
            "compile_success_with_bug(nargo, test_program_dir);",
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
            "compile_failure(nargo, test_program_dir);",
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
    let test_program_dir_path = PathBuf::from("{test_dir}");
    let temp_dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("nargo").unwrap();
    cmd.arg("--program-dir").arg(test_program_dir_path.to_str().unwrap());
    cmd.arg("trace").arg("--trace-dir").arg(temp_dir.path()).arg("--trace-format").arg("json");
    let trace_dir_path = temp_dir.path().as_os_str().to_str().unwrap();
    let trace_file_path = temp_dir.path().join("trace.json");
    let file_written_message = format!("Saved trace to {{trace_dir_path}}");
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

fn generate_interpret_execution_success_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "execution_success";
    let test_cases = read_test_cases(test_data_dir, test_type);

    writeln!(
        test_file,
        "mod interpret_{test_type} {{
        use super::*;
    "
    )
    .unwrap();
    for (test_name, test_dir) in test_cases {
        if IGNORED_INTERPRET_EXECUTION_TESTS.contains(&test_name.as_str()) {
            continue;
        }

        let test_dir = test_dir.display();

        generate_test_cases(
            test_file,
            &test_name,
            &test_dir,
            "interpret",
            "interpret_execution_success(nargo);",
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

fn generate_interpret_execution_failure_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "execution_failure";
    let test_cases = read_test_cases(test_data_dir, test_type);

    writeln!(
        test_file,
        "mod interpret_{test_type} {{
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
            "interpret",
            "interpret_execution_failure(nargo);",
            &MatrixConfig { vary_brillig: true, ..Default::default() },
        );
    }
    writeln!(test_file, "}}").unwrap();
}

/// Run integration tests with the `--minimal-ssa` option and check that the return
/// value matches the expectations. This also enables `--force-brillig` since `--minimal-ssa`
/// is only valid when all functions are unconstrained.
fn generate_minimal_execution_success_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "execution_success";
    let test_cases = read_test_cases(test_data_dir, test_type);

    writeln!(
        test_file,
        "mod minimal_{test_type} {{
        use super::*;
    "
    )
    .unwrap();
    for (test_name, test_dir) in test_cases {
        if IGNORED_MINIMAL_EXECUTION_TESTS.contains(&test_name.as_str()) {
            continue;
        }
        let test_dir = test_dir.display();

        let check_stdout = !TESTS_WITHOUT_STDOUT_CHECK.contains(&test_name.as_str());

        generate_test_cases(
            test_file,
            &test_name,
            &test_dir,
            "execute",
            &format!(
                r#"
                nargo.arg("--minimal-ssa");
                execution_success(nargo, test_program_dir, {check_stdout});
                "#,
            ),
            &MatrixConfig {
                vary_brillig: false,
                vary_inliner: false,
                min_inliner: min_inliner(&test_name),
                max_inliner: max_inliner(&test_name),
            },
        );
    }
    writeln!(test_file, "}}").unwrap();
}

/// Here we check, for every program in `test_programs/execution_success`, that:
/// 1. `nargo expand` works on it
/// 2. That the output of the original program is the same as the output of the expanded program
///    (that is, we run `nargo execute` on the original program and the expanded program and compare the output)
fn generate_nargo_expand_execution_success_tests(test_file: &mut File, test_data_dir: &Path) {
    let test_type = "execution_success";
    let test_cases = read_test_cases(test_data_dir, test_type);

    writeln!(
        test_file,
        "
mod nargo_expand_{test_type} {{
    use super::*;
    "
    )
    .unwrap();

    for (test_name, test_dir) in test_cases {
        if IGNORED_NARGO_EXPAND_EXECUTION_TESTS.contains(&test_name.as_str()) {
            continue;
        }

        let test_dir = test_dir.display();

        write!(
            test_file,
            r#"
    #[test]
    fn test_{test_name}() {{
        let test_program_dir = PathBuf::from("{test_dir}");
        nargo_expand_execute(test_program_dir);
    }}
    "#
        )
        .unwrap();
    }

    writeln!(test_file, "}}").unwrap();
}

/// Here we check, for every program in `test_programs/{test_type}`, that:
/// 1. `nargo expand` works on it
/// 2. Compiling the output works fine
fn generate_nargo_expand_compile_tests(
    test_type: &'static str,
    test_file: &mut File,
    test_data_dir: &Path,
) {
    generate_nargo_expand_compile_tests_with_ignore_list(test_type, test_file, test_data_dir, &[]);
}

fn generate_nargo_expand_compile_tests_with_ignore_list(
    test_type: &'static str,
    test_file: &mut File,
    test_data_dir: &Path,
    ignore: &[&str],
) {
    let test_cases = read_test_cases(test_data_dir, test_type);

    writeln!(
        test_file,
        "
mod nargo_expand_{test_type} {{
    use super::*;
    "
    )
    .unwrap();

    for (test_name, test_dir) in test_cases {
        let should_panic =
            if ignore.contains(&test_name.as_str()) { "#[should_panic]" } else { "" };

        let test_dir = test_dir.display();

        write!(
            test_file,
            r#"
    #[test]
    {should_panic}
    fn test_{test_name}() {{
        let test_program_dir = PathBuf::from("{test_dir}");
        nargo_expand_compile(test_program_dir, "{test_type}");
    }}
    "#
        )
        .unwrap();
    }

    writeln!(test_file, "}}").unwrap();
}
