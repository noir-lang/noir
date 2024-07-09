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

/// Certain features are only available in the elaborator.
/// We skip these tests for non-elaborator code since they are not
/// expected to work there. This can be removed once the old code is removed.
const IGNORED_NEW_FEATURE_TESTS: [&str; 9] = [
    "macros",
    "wildcard_type",
    "type_definition_annotation",
    "numeric_generics_explicit",
    "derive_impl",
    "comptime_traits",
    "comptime_slice_methods",
    "unary_operator_overloading",
    "unquote_multiple_items_from_annotation",
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

        if !IGNORED_NEW_FEATURE_TESTS.contains(&test_name.as_str()) {
            generate_test_case(
                test_file,
                test_type,
                &format!("legacy_{test_name}"),
                &test_dir,
                r#"
                nargo.arg("execute").arg("--force").arg("--use-legacy");
            
                nargo.assert().success();"#,
            );
        }

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

        generate_test_case(
            test_file,
            test_type,
            &format!("legacy_{test_name}"),
            &test_dir,
            r#"
                nargo.arg("execute").arg("--force").arg("--use-legacy");
            
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

        generate_test_case(
            test_file,
            test_type,
            &format!("legacy_{test_name}"),
            &test_dir,
            r#"
        nargo.arg("test").arg("--use-legacy");
        
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

        generate_test_case(
            test_file,
            test_type,
            &format!("legacy_{test_name}"),
            &test_dir,
            r#"
        nargo.arg("test").arg("--use-legacy");
        
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
        let num_opcodes = &json["programs"][0]["functions"][0]["acir_opcodes"];
        assert_eq!(num_opcodes.as_u64().expect("number of opcodes should fit in a u64"), 0);
        "#;

        generate_test_case(
            test_file,
            test_type,
            &test_name,
            &test_dir,
            &format!(
                r#"
                nargo.arg("info").arg("--json").arg("--force");
                
                {assert_zero_opcodes}"#,
            ),
        );

        if !IGNORED_NEW_FEATURE_TESTS.contains(&test_name.as_str()) {
            generate_test_case(
                test_file,
                test_type,
                &format!("legacy_{test_name}"),
                &test_dir,
                &format!(
                    r#"
                nargo.arg("info").arg("--json").arg("--force").arg("--use-legacy");
                
                {assert_zero_opcodes}"#,
                ),
            );
        }
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

        generate_test_case(
            test_file,
            test_type,
            &format!("legacy_{test_name}"),
            &test_dir,
            r#"
        nargo.arg("compile").arg("--force").arg("--use-legacy");
        
        nargo.assert().success();"#,
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

        if !IGNORED_NEW_FEATURE_TESTS.contains(&test_name.as_str()) {
            generate_test_case(
                test_file,
                test_type,
                &format!("legacy_{test_name}"),
                &test_dir,
                r#"
            nargo.arg("compile").arg("--force").arg("--use-legacy");
            
            nargo.assert().failure().stderr(predicate::str::contains("The application panicked (crashed).").not());"#,
            );
        }
    }
}
