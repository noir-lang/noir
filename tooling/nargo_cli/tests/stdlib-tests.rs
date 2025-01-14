//! Execute unit tests in the Noir standard library.
#![allow(clippy::items_after_test_module)]
use clap::Parser;
use fm::FileManager;
use nargo::foreign_calls::DefaultForeignCallBuilder;
use nargo::PrintOutput;
use noirc_driver::{check_crate, file_manager_with_stdlib, CompileOptions};
use noirc_frontend::hir::FunctionNameMatch;
use std::io::Write;
use std::{collections::BTreeMap, path::PathBuf};

use nargo::{
    ops::{report_errors, run_test, TestStatus},
    package::{Package, PackageType},
    parse_all, prepare_package,
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use test_case::test_matrix;

#[derive(Parser, Debug)]
#[command(ignore_errors = true)]
pub struct Options {
    /// Test name to filter for.
    ///
    /// First is assumed to be `run_stdlib_tests` and the second the of the stdlib tests, e.g.:
    ///
    /// ```text
    /// cargo test -p nargo_cli --test stdlib-tests -- run_stdlib_tests sha256
    /// ```
    args: Vec<String>,
}

impl Options {
    pub fn function_name_match(&self) -> FunctionNameMatch {
        match self.args.as_slice() {
            [_test_name, lib] => FunctionNameMatch::Contains(lib.as_str()),
            _ => FunctionNameMatch::Anything,
        }
    }
}

/// Inliner aggressiveness results in different SSA.
/// Inlining happens if `inline_cost - retain_cost < aggressiveness` (see `inlining.rs`).
/// NB the CLI uses maximum aggressiveness.
///
/// Even with the same inlining aggressiveness, forcing Brillig can trigger different behaviour.
#[test_matrix(
    [false, true],
    [i64::MIN, 0, i64::MAX]
)]
fn run_stdlib_tests(force_brillig: bool, inliner_aggressiveness: i64) {
    let opts = Options::parse();

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

    let result = check_crate(&mut context, dummy_crate_id, &Default::default());
    report_errors(result, &context.file_manager, true, false)
        .expect("Error encountered while compiling standard library");

    // We can now search within the stdlib for any test functions to compile.

    let test_functions = context.get_all_test_functions_in_crate_matching(
        context.stdlib_crate_id(),
        opts.function_name_match(),
    );

    let test_report: Vec<(String, TestStatus)> = test_functions
        .into_iter()
        .map(|(test_name, test_function)| {
            let pedantic_solving = true;
            let status = run_test(
                &bn254_blackbox_solver::Bn254BlackBoxSolver(pedantic_solving),
                &mut context,
                &test_function,
                PrintOutput::Stdout,
                &CompileOptions { force_brillig, inliner_aggressiveness, ..Default::default() },
                |output, base| {
                    DefaultForeignCallBuilder::default().with_output(output).build_with_base(base)
                },
            );
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
            TestStatus::Skipped { .. } => {
                writer
                    .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))
                    .expect("Failed to set color");
                writeln!(writer, "skipped").expect("Failed to write to stderr");
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
