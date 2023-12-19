use std::io::Write;

use acvm::BlackBoxFunctionSolver;
use clap::Args;
use nargo::{
    ops::{run_test, TestStatus},
    package::Package,
    prepare_package,
};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{CompileOptions, NOIR_ARTIFACT_VERSION_STRING};
use noirc_frontend::{graph::CrateName, hir::FunctionNameMatch};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::{backends::Backend, cli::check_cmd::check_crate_and_report_errors, errors::CliError};

use super::NargoConfig;

/// Run the tests for this program
#[derive(Debug, Clone, Args)]
pub(crate) struct TestCommand {
    /// If given, only tests with names containing this string will be run
    test_name: Option<String>,

    /// Display output of `println` statements
    #[arg(long)]
    show_output: bool,

    /// Only run tests that match exactly
    #[clap(long)]
    exact: bool,

    /// The name of the package to test
    #[clap(long, conflicts_with = "workspace")]
    package: Option<CrateName>,

    /// Test all packages in the workspace
    #[clap(long, conflicts_with = "package")]
    workspace: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(
    _backend: &Backend,
    args: TestCommand,
    config: NargoConfig,
) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;

    let pattern = match &args.test_name {
        Some(name) => {
            if args.exact {
                FunctionNameMatch::Exact(name)
            } else {
                FunctionNameMatch::Contains(name)
            }
        }
        None => FunctionNameMatch::Anything,
    };

    #[allow(deprecated)]
    let blackbox_solver = barretenberg_blackbox_solver::BarretenbergSolver::new();
    for package in &workspace {
        // By unwrapping here with `?`, we stop the test runner upon a package failing
        // TODO: We should run the whole suite even if there are failures in a package
        run_tests(&blackbox_solver, package, pattern, args.show_output, &args.compile_options)?;
    }

    Ok(())
}

fn run_tests<S: BlackBoxFunctionSolver>(
    blackbox_solver: &S,
    package: &Package,
    fn_name: FunctionNameMatch,
    show_output: bool,
    compile_options: &CompileOptions,
) -> Result<(), CliError> {
    let (mut context, crate_id) = prepare_package(package);
    check_crate_and_report_errors(
        &mut context,
        crate_id,
        compile_options.deny_warnings,
        compile_options.disable_macros,
        compile_options.silence_warnings,
    )?;

    let test_functions = context.get_all_test_functions_in_crate_matching(&crate_id, fn_name);
    let count_all = test_functions.len();
    if count_all == 0 {
        match &fn_name {
            FunctionNameMatch::Exact(pattern) => {
                return Err(CliError::Generic(format!(
                    "[{}] Found 0 tests matching input '{pattern}'.",
                    package.name
                )))
            }
            FunctionNameMatch::Contains(pattern) => {
                return Err(CliError::Generic(format!(
                    "[{}] Found 0 tests containing '{pattern}'.",
                    package.name
                )))
            }
            // If we are running all tests in a crate, having none is not an error
            FunctionNameMatch::Anything => {}
        };
    }

    let plural = if count_all == 1 { "" } else { "s" };
    println!("[{}] Running {count_all} test function{plural}", package.name);
    let mut count_failed = 0;

    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut writer = writer.lock();

    for (test_name, test_function) in test_functions {
        write!(writer, "[{}] Testing {test_name}... ", package.name)
            .expect("Failed to write to stdout");
        writer.flush().expect("Failed to flush writer");

        match run_test(blackbox_solver, &context, test_function, show_output, compile_options) {
            TestStatus::Pass { .. } => {
                writer
                    .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
                    .expect("Failed to set color");
                writeln!(writer, "ok").expect("Failed to write to stdout");
            }
            TestStatus::Fail { message, error_diagnostic } => {
                writer
                    .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                    .expect("Failed to set color");
                writeln!(writer, "{message}\n").expect("Failed to write to stdout");
                if let Some(diag) = error_diagnostic {
                    noirc_errors::reporter::report_all(
                        context.file_manager.as_file_map(),
                        &[diag],
                        compile_options.deny_warnings,
                        compile_options.silence_warnings,
                    );
                }
                count_failed += 1;
            }
            TestStatus::CompileError(err) => {
                noirc_errors::reporter::report_all(
                    context.file_manager.as_file_map(),
                    &[err],
                    compile_options.deny_warnings,
                    compile_options.silence_warnings,
                );
                count_failed += 1;
            }
        }
        writer.reset().expect("Failed to reset writer");
    }

    write!(writer, "[{}] ", package.name).expect("Failed to write to stdout");

    if count_failed == 0 {
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Green))).expect("Failed to set color");
        writeln!(writer, "{count_all} test{plural} passed").expect("Failed to write to stdout");
        writer.reset().expect("Failed to reset writer");

        Ok(())
    } else {
        let count_passed = count_all - count_failed;
        let plural_failed = if count_failed == 1 { "" } else { "s" };
        let plural_passed = if count_passed == 1 { "" } else { "s" };

        if count_passed != 0 {
            writer
                .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
                .expect("Failed to set color");
            write!(writer, "{count_passed} test{plural_passed} passed, ",)
                .expect("Failed to write to stdout");
        }
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Red))).expect("Failed to set color");
        writeln!(writer, "{count_failed} test{plural_failed} failed")
            .expect("Failed to write to stdout");
        writer.reset().expect("Failed to reset writer");

        Err(CliError::Generic(String::new()))
    }
}
