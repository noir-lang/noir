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
    generate_execution_panic_tests(&mut test_file, &test_dir);
    generate_noir_test_success_tests(&mut test_file, &test_dir);
    generate_noir_test_failure_tests(&mut test_file, &test_dir);
    generate_compile_success_empty_tests(&mut test_file, &test_dir);
    generate_compile_success_contract_tests(&mut test_file, &test_dir);
    generate_compile_success_no_bug_tests(&mut test_file, &test_dir);
    generate_compile_success_with_bug_tests(&mut test_file, &test_dir);
    generate_compile_failure_tests(&mut test_file, &test_dir);

    generate_interpret_execution_success_tests(&mut test_file, &test_dir);

    generate_fuzzing_failure_tests(&mut test_file, &test_dir);

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
const INLINER_OVERRIDES: [(&str, i64); 4] = [
    ("reference_counts_inliner_0", 0),
    ("reference_counts_inliner_min", i64::MIN),
    ("reference_counts_inliner_max", i64::MAX),
    ("reference_counts_slices_inliner_0", 0),
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
/// becuase they are too slow to run.
const IGNORED_INTERPRET_EXECUTION_TESTS: [&str; 34] = [
    // slow
    "regression_4709",
    // panic: index out of bounds
    "array_dynamic_nested_blackbox_input",
    // gives a different result with `--force-brillig`
    "array_oob_regression_7965",
    // gives a different result with `--force-brillig`
    "array_oob_regression_7975",
    // wrong result
    "brillig_block_parameter_liveness",
    // panic: BlockArgumentCountMismatch
    "brillig_cow_regression",
    // wrong result
    "databus",
    // panic: index out of bounds
    "databus_composite_calldata",
    // wrong result
    "databus_two_calldata",
    // wrong result
    "databus_two_calldata_simple",
    // panic: IntrinsicArgumentCountMismatch
    "fold_numeric_generic_poseidon",
    // gives a different result with `--force-brillig`
    "global_array_rc_regression_8259",
    // panic: IntrinsicArgumentCountMismatch
    "hash_to_field",
    // panic: Internal(TypeError)
    "inline_decompose_hint_brillig_call",
    // panic: Internal(TypeError)
    "multi_scalar_mul",
    // gives a different result with `--force-brillig`
    "nested_if_then_block_same_cond",
    // panic: IntrinsicArgumentCountMismatch
    "no_predicates_numeric_generic_poseidon",
    // panic: IntrinsicArgumentCountMismatch
    "ram_blowup_regression",
    // panic: index out of bounds
    "regression_11294",
    // panic: Internal(TypeError)
    "regression_3889",
    // panic: IntrinsicArgumentCountMismatch
    "regression_5252",
    // panic: IntrinsicArgumentCountMismatch
    "regression_7128",
    // panic: index out of bounds
    "regression_7612",
    // gives a wrong result
    "regression_7744",
    // gives a wrong result
    "regression_8174",
    // panic: index out of bounds
    "regression_struct_array_conditional",
    // panic Internal(TypeError)
    "simple_shield",
    // panic: index out of bounds
    "slice_loop",
    // panic: index out of bounds
    "struct_array_inputs",
    // panic: BlockArgumentCountMismatch
    "struct_inputs",
    // panic: IntrinsicArgumentCountMismatch
    "to_be_bytes",
    // panic: IntrinsicArgumentCountMismatch
    "to_le_bytes",
    // panic: BlockArgumentCountMismatch
    "tuple_inputs",
    // panic: IntrinsicArgumentCountMismatch
    "unrolling_regression_8333",
];

/// These tests are ignored because making them work involves a more complex test code that
/// might not be worth it.
/// Others are ignored because of existing bugs in `nargo expand`.
/// As the bugs are fixed these tests should be removed from this list.
const IGNORED_NARGO_EXPAND_EXECUTION_TESTS: [&str; 9] = [
    // There's nothing special about this program but making it work with a custom entry would involve
    // having to parse the Nargo.toml file, etc., which is not worth it
    "custom_entry",
    // There's no "src/main.nr" here so it's trickier to make this work
    "diamond_deps_0",
    // There's no "src/main.nr" here so it's trickier to make this work
    "overlapping_dep_and_mod",
    // bug
    "poseidonsponge_x5_254",
    // bug
    "regression_5045",
    // bug
    "regression_7744",
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
const IGNORED_NARGO_EXPAND_COMPILE_SUCCESS_EMPTY_TESTS: [&str; 24] = [
    // bug
    "alias_trait_method_call_multiple_candidates",
    // bug
    "associated_type_bounds",
    // bug
    "enums",
    // There's no "src/main.nr" here so it's trickier to make this work
    "overlapping_dep_and_mod",
    // bug
    "primitive_trait_method_call_multiple_candidates",
    // bug
    "reexports",
    // bug
    "regression_7038",
    // bug
    "regression_7038_2",
    // bug
    "regression_7038_3",
    // bug
    "regression_7038_4",
    // bug
    "serialize_1",
    // bug
    "serialize_2",
    // bug
    "serialize_3",
    // bug
    "serialize_4",
    // bug
    "trait_allowed_item_name_matches",
    // bug
    "trait_default_implementation",
    // bug
    "trait_function_calls",
    // bug
    "trait_method_mut_self",
    // bug
    "trait_override_implementation",
    // bug
    "trait_static_methods",
    // bug
    "type_trait_method_call_multiple_candidates",
    // bug
    "type_trait_method_call_multiple_candidates_with_turbofish",
    // There's no "src/main.nr" here so it's trickier to make this work
    "workspace_reexport_bug",
    // bug
    "nested_trait_associated_type_regression_8252",
];

/// These tests are ignored because of existing bugs in `nargo expand`.
/// As the bugs are fixed these tests should be removed from this list.
const IGNORED_NARGO_EXPAND_COMPILE_SUCCESS_NO_BUG_TESTS: [&str; 18] = [
    "noirc_frontend_tests_arithmetic_generics_checked_casts_do_not_prevent_canonicalization",
    "noirc_frontend_tests_check_trait_as_type_as_fn_parameter",
    "noirc_frontend_tests_check_trait_as_type_as_two_fn_parameters",
    "noirc_frontend_tests_enums_match_on_empty_enum",
    "noirc_frontend_tests_resolves_generic_type_argument_via_self",
    "noirc_frontend_tests_traits_calls_trait_function_if_it_is_in_scope",
    "noirc_frontend_tests_traits_calls_trait_function_if_it_is_only_candidate_in_scope",
    "noirc_frontend_tests_traits_calls_trait_function_if_it_is_only_candidate_in_scope_in_nested_module_using_super",
    "noirc_frontend_tests_traits_serialize_test_with_a_previous_unrelated_definition",
    "noirc_frontend_tests_traits_trait_alias_polymorphic_inheritance",
    "noirc_frontend_tests_traits_trait_alias_single_member",
    "noirc_frontend_tests_traits_trait_alias_two_members",
    "noirc_frontend_tests_traits_trait_impl_with_where_clause_with_trait_with_associated_numeric",
    "noirc_frontend_tests_traits_trait_impl_with_where_clause_with_trait_with_associated_type",
    "noirc_frontend_tests_traits_accesses_associated_type_inside_trait_impl_using_self",
    "noirc_frontend_tests_traits_accesses_associated_type_inside_trait_using_self",
    "noirc_frontend_tests_u32_globals_as_sizes_in_types",
    "noirc_frontend_tests_unused_items_considers_struct_as_constructed_if_trait_method_is_called",
];

const IGNORED_NARGO_EXPAND_COMPILE_SUCCESS_WITH_BUG_TESTS: [&str; 1] =
    ["noirc_frontend_tests_cast_negative_one_to_u8_size_checks"];

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

    let nargo = setup_nargo(&test_program_dir, "{test_command}", force_brillig, inliner_aggressiveness);

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

        generate_test_cases(
            test_file,
            &test_name,
            &test_dir,
            "execute",
            &format!(
                "execution_success(nargo, test_program_dir, {}, force_brillig, inliner_aggressiveness);",
                !TESTS_WITHOUT_STDOUT_CHECK.contains(&test_name.as_str())
            ),
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
            &MatrixConfig::default(),
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
                nargo.assert().failure().stderr(predicate::str::contains("Failing input"));
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
            "compile_success_with_bug(nargo);",
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

/// Here we check, for every program in `test_programs/exeuction_success`, that:
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
        if ignore.contains(&test_name.as_str()) {
            continue;
        }

        let test_dir = test_dir.display();

        write!(
            test_file,
            r#"
    #[test]
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
