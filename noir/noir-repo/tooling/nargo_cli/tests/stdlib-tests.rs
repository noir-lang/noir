use std::io::Write;
use std::{collections::BTreeMap, path::PathBuf};

use fm::FileManager;
use noirc_driver::{check_crate, compile_no_check, file_manager_with_stdlib, CompileOptions};
use noirc_frontend::hir::FunctionNameMatch;

use nargo::{
    ops::{report_errors, run_test, TestStatus},
    package::{Package, PackageType},
    parse_all, prepare_package,
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[test]
fn run_stdlib_tests() {
    let mut file_manager = file_manager_with_stdlib(&PathBuf::from("."));
    file_manager.add_file_with_source_canonical_path(&PathBuf::from("main.nr"), "".to_owned());
    let parsed_files = parse_all(&file_manager);

    // We need a dummy package as we cannot compile the stdlib on its own.
    let dummy_package = Package {
        version: None,
        compiler_required_version: None,
        root_dir: PathBuf::from("."),
        package_type: PackageType::Binary,
        entry_path: PathBuf::from("main.nr"),
        name: "stdlib".parse().unwrap(),
        dependencies: BTreeMap::new(),
        expression_width: None,
    };

    let (mut context, dummy_crate_id) =
        prepare_package(&file_manager, &parsed_files, &dummy_package);

    let result = check_crate(&mut context, dummy_crate_id, false, false, None);
    report_errors(result, &context.file_manager, true, false)
        .expect("Error encountered while compiling standard library");

    // We can now search within the stdlib for any test functions to compile.

    let test_functions = context.get_all_test_functions_in_crate_matching(
        context.stdlib_crate_id(),
        FunctionNameMatch::Anything,
    );

    let test_report: Vec<(String, TestStatus)> = test_functions
        .into_iter()
        .map(|(test_name, test_function)| {
            let test_function_has_no_arguments = context
                .def_interner
                .function_meta(&test_function.get_id())
                .function_signature()
                .0
                .is_empty();

            let status = if test_function_has_no_arguments {
                run_test(
                    &bn254_blackbox_solver::Bn254BlackBoxSolver,
                    &mut context,
                    &test_function,
                    false,
                    None,
                    None,
                    None,
                    &CompileOptions::default(),
                )
            } else {
                use noir_fuzzer::FuzzedExecutor;
                use proptest::test_runner::TestRunner;

                let compiled_program = compile_no_check(
                    &mut context,
                    &CompileOptions::default(),
                    test_function.get_id(),
                    None,
                    false,
                );
                match compiled_program {
                    Ok(compiled_program) => {
                        let runner = TestRunner::default();

                        let fuzzer = FuzzedExecutor::new(compiled_program.into(), runner);

                        let result = fuzzer.fuzz();
                        if result.success {
                            TestStatus::Pass
                        } else {
                            TestStatus::Fail {
                                message: result.reason.unwrap_or_default(),
                                error_diagnostic: None,
                            }
                        }
                    }
                    Err(err) => TestStatus::CompileError(err.into()),
                }
            };
            (test_name, status)
        })
        .collect();

    assert!(!test_report.is_empty(), "Could not find any tests within the stdlib");
    display_test_report(&file_manager, &dummy_package, &CompileOptions::default(), &test_report);
    assert!(test_report.iter().all(|(_, status)| !status.failed()));
}

// This code is copied from `src/cli/test_cmd.rs`.
// This should be abstracted into a proper test runner at some point.
fn display_test_report(
    file_manager: &FileManager,
    package: &Package,
    compile_options: &CompileOptions,
    test_report: &[(String, TestStatus)],
) {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut writer = writer.lock();

    for (test_name, test_status) in test_report {
        write!(writer, "[{}] Testing {test_name}... ", package.name)
            .expect("Failed to write to stderr");
        writer.flush().expect("Failed to flush writer");

        match &test_status {
            TestStatus::Pass { .. } => {
                writer
                    .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
                    .expect("Failed to set color");
                writeln!(writer, "ok").expect("Failed to write to stderr");
            }
            TestStatus::Fail { message, error_diagnostic } => {
                writer
                    .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                    .expect("Failed to set color");
                writeln!(writer, "FAIL\n{message}\n").expect("Failed to write to stderr");
                if let Some(diag) = error_diagnostic {
                    noirc_errors::reporter::report_all(
                        file_manager.as_file_map(),
                        &[diag.clone()],
                        compile_options.deny_warnings,
                        compile_options.silence_warnings,
                    );
                }
            }
            TestStatus::CompileError(err) => {
                noirc_errors::reporter::report_all(
                    file_manager.as_file_map(),
                    &[err.clone()],
                    compile_options.deny_warnings,
                    compile_options.silence_warnings,
                );
            }
        }
        writer.reset().expect("Failed to reset writer");
    }

    write!(writer, "[{}] ", package.name).expect("Failed to write to stderr");

    let count_all = test_report.len();
    let count_failed = test_report.iter().filter(|(_, status)| status.failed()).count();
    let plural = if count_all == 1 { "" } else { "s" };
    if count_failed == 0 {
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Green))).expect("Failed to set color");
        write!(writer, "{count_all} test{plural} passed").expect("Failed to write to stderr");
        writer.reset().expect("Failed to reset writer");
        writeln!(writer).expect("Failed to write to stderr");
    } else {
        let count_passed = count_all - count_failed;
        let plural_failed = if count_failed == 1 { "" } else { "s" };
        let plural_passed = if count_passed == 1 { "" } else { "s" };

        if count_passed != 0 {
            writer
                .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
                .expect("Failed to set color");
            write!(writer, "{count_passed} test{plural_passed} passed, ",)
                .expect("Failed to write to stderr");
        }

        writer.set_color(ColorSpec::new().set_fg(Some(Color::Red))).expect("Failed to set color");
        writeln!(writer, "{count_failed} test{plural_failed} failed")
            .expect("Failed to write to stderr");
        writer.reset().expect("Failed to reset writer");
    }
}
