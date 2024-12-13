use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    panic::{catch_unwind, UnwindSafe},
    path::PathBuf,
    sync::{mpsc, Mutex},
    thread,
    time::Duration,
};

use acvm::{BlackBoxFunctionSolver, FieldElement};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;
use fm::FileManager;
use formatters::{Formatter, PrettyFormatter, TerseFormatter};
use nargo::{
    insert_all_files_for_workspace_into_file_manager, ops::TestStatus, package::Package, parse_all,
    prepare_package, workspace::Workspace, PrintOutput,
};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml};
use noirc_driver::{check_crate, CompileOptions, NOIR_ARTIFACT_VERSION_STRING};
use noirc_frontend::hir::{FunctionNameMatch, ParsedFiles};

use crate::{cli::check_cmd::check_crate_and_report_errors, errors::CliError};

use super::{NargoConfig, PackageOptions};

mod formatters;

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
    #[clap(long, default_value_t = rayon::current_num_threads())]
    test_threads: usize,

    /// Configure formatting of output
    #[clap(long)]
    format: Option<Format>,

    /// Display one character per test instead of one line
    #[clap(short = 'q', long = "quiet")]
    quiet: bool,
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
enum Format {
    /// Print verbose output
    Pretty,
    /// Display one character per test
    Terse,
}

impl Format {
    fn formatter(&self) -> Box<dyn Formatter> {
        match self {
            Format::Pretty => Box::new(PrettyFormatter),
            Format::Terse => Box::new(TerseFormatter),
        }
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Pretty => write!(f, "pretty"),
            Format::Terse => write!(f, "terse"),
        }
    }
}

struct Test<'a> {
    name: String,
    package_name: String,
    runner: Box<dyn FnOnce() -> (TestStatus, String) + Send + UnwindSafe + 'a>,
}

struct TestResult {
    name: String,
    package_name: String,
    status: TestStatus,
    output: String,
    time_to_run: Duration,
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

    let formatter: Box<dyn Formatter> = if let Some(format) = args.format {
        format.formatter()
    } else if args.quiet {
        Box::new(TerseFormatter)
    } else {
        Box::new(PrettyFormatter)
    };

    let runner = TestRunner {
        file_manager: &file_manager,
        parsed_files: &parsed_files,
        workspace,
        args: &args,
        pattern,
        num_threads: args.test_threads,
        formatter,
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
    formatter: Box<dyn Formatter>,
}

impl<'a> TestRunner<'a> {
    fn run(&self) -> Result<(), CliError> {
        // First compile all packages and collect their tests
        let packages_tests = self.collect_packages_tests()?;

        // Now gather all tests and how many are per packages
        let mut tests = Vec::new();
        let mut test_count_per_package = BTreeMap::new();

        for (package_name, package_tests) in packages_tests {
            test_count_per_package.insert(package_name, package_tests.len());
            tests.extend(package_tests);
        }

        // Now run all tests in parallel, but show output for each package sequentially
        let tests_count = tests.len();
        let all_passed = self.run_all_tests(tests, &test_count_per_package);

        if tests_count == 0 {
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

        if all_passed {
            Ok(())
        } else {
            Err(CliError::Generic(String::new()))
        }
    }

    /// Runs all tests. Returns `true` if all tests passed, `false` otherwise.
    fn run_all_tests(
        &self,
        tests: Vec<Test<'a>>,
        test_count_per_package: &BTreeMap<String, usize>,
    ) -> bool {
        let mut all_passed = true;

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

                        let time_before_test = std::time::Instant::now();
                        let (status, output) = match catch_unwind(test.runner) {
                            Ok((status, output)) => (status, output),
                            Err(err) => (
                                TestStatus::Fail {
                                    message:
                                        // It seems `panic!("...")` makes the error be `&str`, so we handle this common case
                                        if let Some(message) = err.downcast_ref::<&str>() {
                                            message.to_string()
                                        } else {
                                            "An unexpected error happened".to_string()
                                        },
                                    error_diagnostic: None,
                                },
                                String::new(),
                            ),
                        };
                        let time_to_run = time_before_test.elapsed();

                        let test_result = TestResult {
                            name: test.name,
                            package_name: test.package_name,
                            status,
                            output,
                            time_to_run,
                        };

                        if thread_sender.send(test_result).is_err() {
                            break;
                        }
                    })
                    .unwrap();
            }

            // Also drop main sender so the channel closes
            drop(sender);

            // We'll go package by package, but we might get test results from packages ahead of us.
            // We'll buffer those here and show them all at once when we get to those packages.
            let mut buffer: HashMap<String, Vec<TestResult>> = HashMap::new();
            for (package_name, total_test_count) in test_count_per_package {
                let mut test_report = Vec::new();

                let mut current_test_count = 0;
                let total_test_count = *total_test_count;

                self.formatter
                    .package_start(package_name, total_test_count)
                    .expect("Could not display package start");

                // Check if we have buffered test results for this package
                if let Some(buffered_tests) = buffer.remove(package_name) {
                    for test_result in buffered_tests {
                        self.display_test_result(
                            &test_result,
                            current_test_count + 1,
                            total_test_count,
                        )
                        .expect("Could not display test status");
                        test_report.push(test_result);
                        current_test_count += 1;
                    }
                }

                if current_test_count < total_test_count {
                    while let Ok(test_result) = receiver.recv() {
                        if test_result.status.failed() {
                            all_passed = false;
                        }

                        // This is a test result from a different package: buffer it.
                        if &test_result.package_name != package_name {
                            buffer
                                .entry(test_result.package_name.clone())
                                .or_default()
                                .push(test_result);
                            continue;
                        }

                        self.display_test_result(
                            &test_result,
                            current_test_count + 1,
                            total_test_count,
                        )
                        .expect("Could not display test status");
                        test_report.push(test_result);
                        current_test_count += 1;
                        if current_test_count == total_test_count {
                            break;
                        }
                    }
                }

                self.formatter
                    .package_end(
                        package_name,
                        &test_report,
                        self.file_manager,
                        self.args.show_output,
                        self.args.compile_options.deny_warnings,
                        self.args.compile_options.silence_warnings,
                    )
                    .expect("Could not display test report");
            }
        });

        all_passed
    }

    /// Compiles all packages in parallel and returns their tests
    fn collect_packages_tests(&'a self) -> Result<BTreeMap<String, Vec<Test<'a>>>, CliError> {
        let mut package_tests = BTreeMap::new();
        let mut error = None;

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
                        if thread_sender.send((package, tests)).is_err() {
                            break;
                        }
                    })
                    .unwrap();
            }

            // Also drop main sender so the channel closes
            drop(sender);

            for (package, tests) in receiver.iter() {
                match tests {
                    Ok(tests) => {
                        package_tests.insert(package.name.to_string(), tests);
                    }
                    Err(err) => {
                        error = Some(err);
                    }
                }
            }
        });

        if let Some(error) = error {
            Err(error)
        } else {
            Ok(package_tests)
        }
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
    fn display_test_result(
        &'a self,
        test_result: &'a TestResult,
        current_test_count: usize,
        total_test_count: usize,
    ) -> std::io::Result<()> {
        self.formatter.test_end(
            test_result,
            current_test_count,
            total_test_count,
            self.file_manager,
            self.args.show_output,
            self.args.compile_options.deny_warnings,
            self.args.compile_options.silence_warnings,
        )
    }
}
