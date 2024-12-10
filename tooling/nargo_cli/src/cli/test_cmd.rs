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
use fm::FileManager;
use nargo::{
    insert_all_files_for_workspace_into_file_manager, ops::TestStatus, package::Package, parse_all,
    prepare_package, workspace::Workspace, PrintOutput,
};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml};
use noirc_driver::{check_crate, CompileOptions, NOIR_ARTIFACT_VERSION_STRING};
use noirc_frontend::hir::{FunctionNameMatch, ParsedFiles};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::{cli::check_cmd::check_crate_and_report_errors, errors::CliError};

use super::{NargoConfig, PackageOptions};

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

    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    compile_options: CompileOptions,

    /// JSON RPC url to solve oracle calls
    #[clap(long)]
    oracle_resolver: Option<String>,

    /// Number of threads used for running tests in parallel
    #[clap(long)]
    test_threads: Option<usize>,
}

struct Test<'a> {
    name: String,
    package_name: String,
    runner: Box<dyn FnOnce() -> (TestStatus, String) + Send + 'a>,
}

const STACK_SIZE: usize = 4 * 1024 * 1024;

pub(crate) fn run(args: TestCommand, config: NargoConfig) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let selection = args.package_options.package_selection();
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;

    let mut file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut file_manager);
    let parsed_files = parse_all(&file_manager);

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

    let num_threads = args.test_threads.unwrap_or_else(rayon::current_num_threads);

    let runner = TestRunner {
        file_manager: &file_manager,
        parsed_files: &parsed_files,
        workspace,
        args: &args,
        pattern,
        num_threads,
    };
    runner.run()
}

struct TestRunner<'a> {
    file_manager: &'a FileManager,
    parsed_files: &'a ParsedFiles,
    workspace: Workspace,
    args: &'a TestCommand,
    pattern: FunctionNameMatch<'a>,
    num_threads: usize,
}

impl<'a> TestRunner<'a> {
    fn run(&self) -> Result<(), CliError> {
        // First compile all packages and collect their tests
        let package_test_results = self.collect_packages_tests();

        // Now gather all tests and how many are per packages
        let mut tests = Vec::new();
        let mut test_count_per_package = HashMap::new();

        for (package_name, package_tests) in package_test_results {
            let package_tests = package_tests?;
            test_count_per_package.insert(package_name, package_tests.len());
            tests.extend(package_tests);
        }

        // If there were no compile errors we'll run them all. Here we show how many there are per package.
        for (package_name, test_count) in &test_count_per_package {
            let plural = if *test_count == 1 { "" } else { "s" };
            println!("[{package_name}] Running {test_count} test function{plural}");
        }

        // Now run all tests in parallel, but show output for each package sequentially
        let test_reports = self.run_all_tests(tests, &test_count_per_package);

        if test_reports.is_empty() {
            match &self.pattern {
                FunctionNameMatch::Exact(pattern) => {
                    return Err(CliError::Generic(format!(
                        "Found 0 tests matching input '{pattern}'.",
                    )))
                }
                FunctionNameMatch::Contains(pattern) => {
                    return Err(CliError::Generic(
                        format!("Found 0 tests containing '{pattern}'.",),
                    ))
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

    fn run_all_tests(
        &self,
        tests: Vec<Test<'a>>,
        test_count_per_package: &HashMap<String, usize>,
    ) -> Vec<(String, TestStatus)> {
        // Here we'll gather all test reports from all packages
        let mut test_reports = Vec::new();

        let (sender, receiver) = mpsc::channel();
        let iter = &Mutex::new(tests.into_iter());
        thread::scope(|scope| {
            // Start worker threads
            for _ in 0..self.num_threads {
                // Clone sender so it's dropped once the thread finishes
                let thread_sender = sender.clone();
                thread::Builder::new()
                    // Specify a larger-than-default stack size to prevent overflowing stack in large programs.
                    // (the default is 2MB)
                    .stack_size(STACK_SIZE)
                    .spawn_scoped(scope, move || loop {
                        // Get next test to process from the iterator.
                        let Some(test) = iter.lock().unwrap().next() else {
                            break;
                        };
                        let test_status = (test.runner)();

                        // It's fine to ignore the result of sending.
                        // If the receiver has hung up, everything will wind down soon anyway.
                        let _ = thread_sender.send((test.name, test.package_name, test_status));
                    })
                    .unwrap();
            }

            // Also drop main sender so the channel closes
            drop(sender);

            // We'll go package by package, but we might get test results from packages ahead of us.
            // We'll buffer those here and show them all at once when we get to those packages.
            let mut buffer: HashMap<String, Vec<(String, TestStatus, String)>> = HashMap::new();
            for (package_name, test_count) in test_count_per_package {
                let mut test_report = Vec::new();

                // How many tests are left to receive for this package
                let mut remaining_test_count = *test_count;

                // Check if we have buffered test results for this package
                if let Some(buffered_tests) = buffer.remove(package_name) {
                    remaining_test_count -= buffered_tests.len();

                    for (test_name, test_status, output) in buffered_tests {
                        self.display_test_status(&test_name, package_name, &test_status, output)
                            .expect("Could not display test status");
                        test_report.push((test_name, test_status));
                    }
                }

                if remaining_test_count > 0 {
                    while let Ok((test_name, test_package_name, (test_status, output))) =
                        receiver.recv()
                    {
                        // This is a test result from a different package: buffer it.
                        if &test_package_name != package_name {
                            buffer.entry(test_package_name).or_default().push((
                                test_name,
                                test_status,
                                output,
                            ));
                            continue;
                        }

                        self.display_test_status(
                            &test_name,
                            &test_package_name,
                            &test_status,
                            output,
                        )
                        .expect("Could not display test status");
                        test_report.push((test_name, test_status));
                        remaining_test_count -= 1;
                        if remaining_test_count == 0 {
                            break;
                        }
                    }
                }

                display_test_report(package_name, &test_report)
                    .expect("Could not display test report");
                test_reports.extend(test_report);
            }
        });

        test_reports
    }

    /// Compiles all packages in parallel and returns their tests
    fn collect_packages_tests(&'a self) -> HashMap<String, Result<Vec<Test<'a>>, CliError>> {
        let mut package_tests = HashMap::new();

        let (sender, receiver) = mpsc::channel();
        let iter = &Mutex::new(self.workspace.into_iter());

        thread::scope(|scope| {
            // Start worker threads
            for _ in 0..self.num_threads {
                // Clone sender so it's dropped once the thread finishes
                let thread_sender = sender.clone();
                thread::Builder::new()
                    // Specify a larger-than-default stack size to prevent overflowing stack in large programs.
                    // (the default is 2MB)
                    .stack_size(STACK_SIZE)
                    .spawn_scoped(scope, move || loop {
                        // Get next package to process from the iterator.
                        let Some(package) = iter.lock().unwrap().next() else {
                            break;
                        };
                        let tests = self.collect_package_tests::<Bn254BlackBoxSolver>(
                            package,
                            self.args.oracle_resolver.as_deref(),
                            Some(self.workspace.root_dir.clone()),
                            package.name.to_string(),
                        );
                        let _ = thread_sender.send((package, tests));
                    })
                    .unwrap();
            }

            // Also drop main sender so the channel closes
            drop(sender);

            for (package, tests) in receiver.iter() {
                package_tests.insert(package.name.to_string(), tests);
            }
        });

        package_tests
    }

    /// Compiles a single package and returns all of its tests
    fn collect_package_tests<S: BlackBoxFunctionSolver<FieldElement> + Default>(
        &'a self,
        package: &'a Package,
        foreign_call_resolver_url: Option<&'a str>,
        root_path: Option<PathBuf>,
        package_name: String,
    ) -> Result<Vec<Test<'a>>, CliError> {
        let test_functions = self.get_tests_in_package(package)?;

        let tests: Vec<Test> = test_functions
            .into_iter()
            .map(|test_name| {
                let test_name_copy = test_name.clone();
                let root_path = root_path.clone();
                let package_name_clone = package_name.clone();
                let package_name_clone2 = package_name.clone();
                let runner = Box::new(move || {
                    self.run_test::<S>(
                        package,
                        &test_name,
                        foreign_call_resolver_url,
                        root_path,
                        package_name_clone.clone(),
                    )
                });
                Test { name: test_name_copy, package_name: package_name_clone2, runner }
            })
            .collect();

        Ok(tests)
    }

    /// Compiles a single package and returns all of its test names
    fn get_tests_in_package(&'a self, package: &'a Package) -> Result<Vec<String>, CliError> {
        let (mut context, crate_id) =
            prepare_package(self.file_manager, self.parsed_files, package);
        check_crate_and_report_errors(&mut context, crate_id, &self.args.compile_options)?;

        Ok(context
            .get_all_test_functions_in_crate_matching(&crate_id, self.pattern)
            .into_iter()
            .map(|(test_name, _)| test_name)
            .collect())
    }

    /// Runs a single test and returns its status together with whatever was printed to stdout
    /// during the test.
    fn run_test<S: BlackBoxFunctionSolver<FieldElement> + Default>(
        &'a self,
        package: &Package,
        fn_name: &str,
        foreign_call_resolver_url: Option<&str>,
        root_path: Option<PathBuf>,
        package_name: String,
    ) -> (TestStatus, String) {
        // This is really hacky but we can't share `Context` or `S` across threads.
        // We then need to construct a separate copy for each test.

        let (mut context, crate_id) =
            prepare_package(self.file_manager, self.parsed_files, package);
        check_crate(&mut context, crate_id, &self.args.compile_options)
            .expect("Any errors should have occurred when collecting test functions");

        let test_functions = context
            .get_all_test_functions_in_crate_matching(&crate_id, FunctionNameMatch::Exact(fn_name));
        let (_, test_function) = test_functions.first().expect("Test function should exist");

        let blackbox_solver = S::default();
        let mut output_string = String::new();

        let test_status = nargo::ops::run_test(
            &blackbox_solver,
            &mut context,
            test_function,
            PrintOutput::String(&mut output_string),
            foreign_call_resolver_url,
            root_path,
            Some(package_name),
            &self.args.compile_options,
        );
        (test_status, output_string)
    }

    /// Display the status of a single test
    fn display_test_status(
        &'a self,
        test_name: &'a String,
        package_name: &'a String,
        test_status: &'a TestStatus,
        output: String,
    ) -> std::io::Result<()> {
        let writer = StandardStream::stderr(ColorChoice::Always);
        let mut writer = writer.lock();

        write!(writer, "[{}] Testing {test_name}... ", package_name)?;
        writer.flush()?;

        match &test_status {
            TestStatus::Pass { .. } => {
                writer.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                writeln!(writer, "ok")?;
            }
            TestStatus::Fail { message, error_diagnostic } => {
                writer.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
                writeln!(writer, "FAIL\n{message}\n")?;
                if let Some(diag) = error_diagnostic {
                    noirc_errors::reporter::report_all(
                        self.file_manager.as_file_map(),
                        &[diag.clone()],
                        self.args.compile_options.deny_warnings,
                        self.args.compile_options.silence_warnings,
                    );
                }
            }
            TestStatus::Skipped { .. } => {
                writer.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                writeln!(writer, "skipped")?;
            }
            TestStatus::CompileError(err) => {
                noirc_errors::reporter::report_all(
                    self.file_manager.as_file_map(),
                    &[err.clone()],
                    self.args.compile_options.deny_warnings,
                    self.args.compile_options.silence_warnings,
                );
            }
        }
        writer.reset()?;

        if self.args.show_output {
            write!(writer, "{output}")
        } else {
            Ok(())
        }
    }
}

/// Display a report for all tests in a package
fn display_test_report(
    package_name: &String,
    test_report: &[(String, TestStatus)],
) -> std::io::Result<()> {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut writer = writer.lock();

    let failed_tests: Vec<_> = test_report
        .iter()
        .filter_map(|(name, status)| if status.failed() { Some(name) } else { None })
        .collect();

    if !failed_tests.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "[{}] Failures:", package_name)?;
        for failed_test in failed_tests {
            writeln!(writer, "     {}", failed_test)?;
        }
        writeln!(writer)?;
    }

    write!(writer, "[{}] ", package_name)?;

    let count_all = test_report.len();
    let count_failed = test_report.iter().filter(|(_, status)| status.failed()).count();
    let plural = if count_all == 1 { "" } else { "s" };
    if count_failed == 0 {
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        write!(writer, "{count_all} test{plural} passed")?;
        writer.reset()?;
        writeln!(writer)?;
    } else {
        let count_passed = count_all - count_failed;
        let plural_failed = if count_failed == 1 { "" } else { "s" };
        let plural_passed = if count_passed == 1 { "" } else { "s" };

        if count_passed != 0 {
            writer.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            write!(writer, "{count_passed} test{plural_passed} passed, ")?;
        }

        writer.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
        writeln!(writer, "{count_failed} test{plural_failed} failed")?;
        writer.reset()?;
    }

    Ok(())
}
