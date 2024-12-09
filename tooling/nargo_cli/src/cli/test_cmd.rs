use std::{
    collections::HashMap,
    io::Write,
    path::PathBuf,
    sync::{mpsc, Mutex},
    thread,
};

use acvm::{BlackBoxFunctionSolver, FieldElement};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;
use fm::{FileId, FileManager};
use nargo::{
    insert_all_files_for_workspace_into_file_manager,
    ops::TestStatus,
    package::{CrateName, Package},
    parse_all, prepare_package,
};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{check_crate, CompileOptions, NOIR_ARTIFACT_VERSION_STRING};
use noirc_frontend::{
    hir::{FunctionNameMatch, ParsedFiles},
    parser::ParserError,
    ParsedModule,
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::{cli::check_cmd::check_crate_and_report_errors, errors::CliError};

use super::NargoConfig;

/// Run the tests for this program
#[derive(Debug, Clone, Args)]
#[clap(visible_alias = "t")]
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

    /// JSON RPC url to solve oracle calls
    #[clap(long)]
    oracle_resolver: Option<String>,
}

struct Test<'a> {
    name: String,
    runner: Box<dyn FnOnce() -> TestStatus + Send + 'a>,
}

pub(crate) fn run(args: TestCommand, config: NargoConfig) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.clone().map_or(default_selection, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;

    let mut workspace_file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = parse_all(&workspace_file_manager);

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

    let num_threads = std::thread::available_parallelism().ok().map(Into::into).unwrap_or(1);
    let mut test_reports = Vec::new();

    for package in workspace.into_iter() {
        let test_report = run_package_tests(
            package,
            &workspace_file_manager,
            &parsed_files,
            pattern,
            &args,
            &workspace,
            num_threads,
        )?;
        test_reports.extend(test_report);
    }

    if test_reports.is_empty() {
        match &pattern {
            FunctionNameMatch::Exact(pattern) => {
                return Err(CliError::Generic(
                    format!("Found 0 tests matching input '{pattern}'.",),
                ))
            }
            FunctionNameMatch::Contains(pattern) => {
                return Err(CliError::Generic(format!("Found 0 tests containing '{pattern}'.",)))
            }
            // If we are running all tests in a crate, having none is not an error
            FunctionNameMatch::Anything => {}
        };
    }

    if test_reports.iter().any(|(_, status)| status.failed()) {
        Err(CliError::Generic(String::new()))
    } else {
        Ok(())
    }
}

fn run_package_tests(
    package: &Package,
    workspace_file_manager: &FileManager,
    parsed_files: &HashMap<FileId, (ParsedModule, Vec<ParserError>)>,
    pattern: FunctionNameMatch<'_>,
    args: &TestCommand,
    workspace: &nargo::workspace::Workspace,
    num_threads: usize,
) -> Result<Vec<(String, TestStatus)>, CliError> {
    let package_name = package.name.to_string();
    let tests = collect_package_tests::<Bn254BlackBoxSolver>(
        workspace_file_manager,
        parsed_files,
        package,
        pattern,
        args.show_output,
        args.oracle_resolver.as_deref(),
        Some(workspace.root_dir.clone()),
        package_name.clone(),
        &args.compile_options,
    )?;
    let num_tests = tests.len();

    let (sender, receiver) = mpsc::channel();
    let iter = Mutex::new(tests.into_iter());
    thread::scope(|scope| {
        // Start worker threads
        for _ in 0..num_threads {
            // Specify a larger-than-default stack size to prevent overflowing stack in large programs.
            // (the default is 2MB)
            thread::Builder::new()
                .stack_size(4 * 1024 * 1024)
                .spawn_scoped(scope, || loop {
                    // Get next test to process from the iterator.
                    let Some(test) = iter.lock().unwrap().next() else {
                        break;
                    };
                    let test_status = (test.runner)();

                    // It's fine to ignore the result of sending. If the
                    // receiver has hung up, everything will wind down soon
                    // anyway.
                    let _ = sender.send((test.name, test_status));
                })
                .unwrap();
        }
    });

    let mut test_report = Vec::new();
    for (test_name, test_status) in receiver.iter().take(num_tests) {
        test_report.push((test_name, test_status));
    }

    display_test_report(&package_name, &test_report)?;

    Ok(test_report)
}

#[allow(clippy::too_many_arguments)]
fn collect_package_tests<'a, S: BlackBoxFunctionSolver<FieldElement> + Default>(
    file_manager: &'a FileManager,
    parsed_files: &'a ParsedFiles,
    package: &'a Package,
    fn_name: FunctionNameMatch,
    show_output: bool,
    foreign_call_resolver_url: Option<&'a str>,
    root_path: Option<PathBuf>,
    package_name: String,
    compile_options: &'a CompileOptions,
) -> Result<Vec<Test<'a>>, CliError> {
    let test_functions =
        get_tests_in_package(file_manager, parsed_files, package, fn_name, compile_options)?;

    let count_all = test_functions.len();

    let plural = if count_all == 1 { "" } else { "s" };
    println!("[{}] Running {count_all} test function{plural}", package.name);

    let tests: Vec<Test> = test_functions
        .into_iter()
        .map(|test_name| {
            let test_name_copy = test_name.clone();
            let root_path = root_path.clone();
            let package_name_clone = package_name.clone();
            let runner = Box::new(move || {
                let test_status = run_test::<S>(
                    file_manager,
                    parsed_files,
                    package,
                    &test_name,
                    show_output,
                    foreign_call_resolver_url,
                    root_path,
                    package_name_clone.clone(),
                    compile_options,
                );
                display_test_status(
                    &test_name,
                    &package_name_clone,
                    &test_status,
                    file_manager,
                    compile_options,
                );
                test_status
            });
            Test { name: test_name_copy, runner }
        })
        .collect();

    Ok(tests)
}

#[allow(clippy::too_many_arguments)]
fn run_test<S: BlackBoxFunctionSolver<FieldElement> + Default>(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    fn_name: &str,
    show_output: bool,
    foreign_call_resolver_url: Option<&str>,
    root_path: Option<PathBuf>,
    package_name: String,
    compile_options: &CompileOptions,
) -> TestStatus {
    // This is really hacky but we can't share `Context` or `S` across threads.
    // We then need to construct a separate copy for each test.

    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);
    check_crate(&mut context, crate_id, compile_options)
        .expect("Any errors should have occurred when collecting test functions");

    let test_functions = context
        .get_all_test_functions_in_crate_matching(&crate_id, FunctionNameMatch::Exact(fn_name));
    let (_, test_function) = test_functions.first().expect("Test function should exist");

    let blackbox_solver = S::default();

    nargo::ops::run_test(
        &blackbox_solver,
        &mut context,
        test_function,
        show_output,
        foreign_call_resolver_url,
        root_path,
        Some(package_name),
        compile_options,
    )
}

fn get_tests_in_package(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    fn_name: FunctionNameMatch,
    options: &CompileOptions,
) -> Result<Vec<String>, CliError> {
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);
    check_crate_and_report_errors(&mut context, crate_id, options)?;

    Ok(context
        .get_all_test_functions_in_crate_matching(&crate_id, fn_name)
        .into_iter()
        .map(|(test_name, _)| test_name)
        .collect())
}

fn display_test_status(
    test_name: &String,
    package_name: &String,
    test_status: &TestStatus,
    file_manager: &FileManager,
    compile_options: &CompileOptions,
) {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut writer = writer.lock();

    write!(writer, "[{}] Testing {test_name}... ", package_name)
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

fn display_test_report(
    package_name: &String,
    test_report: &[(String, TestStatus)],
) -> Result<(), CliError> {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut writer = writer.lock();

    write!(writer, "[{}] ", package_name).expect("Failed to write to stderr");

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

    Ok(())
}
