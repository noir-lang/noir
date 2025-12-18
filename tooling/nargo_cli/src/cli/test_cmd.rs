use std::{
    cmp::max,
    collections::{BTreeMap, HashMap},
    fmt::Display,
    panic::{UnwindSafe, catch_unwind},
    path::PathBuf,
    sync::{
        Mutex,
        mpsc::{self, Sender},
    },
    thread,
    time::Duration,
};

use acvm::{BlackBoxFunctionSolver, FieldElement};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;
use fm::FileManager;
use formatters::{Formatter, JsonFormatter, PrettyFormatter, TerseFormatter};
use nargo::{
    FuzzExecutionConfig, FuzzFolderConfig,
    foreign_calls::DefaultForeignCallBuilder,
    insert_all_files_for_workspace_into_file_manager,
    ops::{FuzzConfig, TestStatus, check_crate_and_report_errors},
    package::Package,
    parse_all, prepare_package,
    workspace::Workspace,
};
use nargo_toml::PackageSelection;
use noirc_driver::{CompileOptions, check_crate};
use noirc_frontend::hir::{FunctionNameMatch, ParsedFiles, def_map::TestFunction};

use crate::errors::CliError;

use super::{LockType, PackageOptions, WorkspaceCommand};

pub(crate) mod formatters;

/// Run the tests for this program
#[derive(Debug, Clone, Args)]
#[clap(visible_alias = "t")]
pub(crate) struct TestCommand {
    /// If given, only tests with names containing this string will be run
    test_names: Vec<String>,

    /// Display output of `println` statements
    #[arg(long)]
    show_output: bool,

    /// Only run tests that match exactly
    #[clap(long)]
    exact: bool,

    /// Print all matching test names, without running them.
    #[clap(long)]
    vector_tests: bool,

    /// Only compile the tests, without running them.
    #[clap(long)]
    no_run: bool,

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

    /// Do not run fuzz tests (tests that have arguments)
    #[clap(long, conflicts_with("only_fuzz"))]
    no_fuzz: bool,

    /// Only run fuzz tests (tests that have arguments)
    #[clap(long, conflicts_with("no_fuzz"))]
    only_fuzz: bool,

    /// If given, load/store fuzzer corpus from this folder
    #[arg(long)]
    corpus_dir: Option<String>,

    /// If given, perform corpus minimization instead of fuzzing and store results in the given folder
    #[arg(long)]
    minimized_corpus_dir: Option<String>,

    /// If given, store the failing input in the given folder
    #[arg(long)]
    fuzzing_failure_dir: Option<String>,

    /// Maximum time in seconds to spend fuzzing (default: 1 seconds)
    #[arg(long, default_value_t = 1)]
    fuzz_timeout: u64,

    /// Maximum number of executions to run for each fuzz test (default: 100000)
    #[arg(long, default_value_t = 100000)]
    fuzz_max_executions: usize,

    /// Show progress of fuzzing (default: false)
    #[arg(long)]
    fuzz_show_progress: bool,
}

impl WorkspaceCommand for TestCommand {
    fn package_selection(&self) -> PackageSelection {
        self.package_options.package_selection()
    }
    fn lock_type(&self) -> LockType {
        // Reads the code to compile tests in memory, but doesn't save artifacts.
        LockType::None
    }
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
enum Format {
    /// Print verbose output
    Pretty,
    /// Display one character per test
    Terse,
    /// Output a JSON Lines document
    Json,
}

impl Format {
    fn formatter(&self) -> Box<dyn Formatter> {
        match self {
            Format::Pretty => Box::new(PrettyFormatter),
            Format::Terse => Box::new(TerseFormatter),
            Format::Json => Box::new(JsonFormatter),
        }
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Pretty => write!(f, "pretty"),
            Format::Terse => write!(f, "terse"),
            Format::Json => write!(f, "json"),
        }
    }
}

struct Test<'a> {
    name: String,
    package_name: String,
    has_arguments: bool,
    runner: Box<dyn FnOnce() -> (TestStatus, String) + Send + UnwindSafe + 'a>,
}

pub(crate) struct TestResult {
    name: String,
    package_name: String,
    status: TestStatus,
    output: String,
    time_to_run: Duration,
}

impl TestResult {
    pub(crate) fn new(
        name: String,
        package_name: String,
        status: TestStatus,
        output: String,
        time_to_run: Duration,
    ) -> Self {
        TestResult { name, package_name, status, output, time_to_run }
    }
}

const STACK_SIZE: usize = 4 * 1024 * 1024;

pub(crate) fn run(args: TestCommand, workspace: Workspace) -> Result<(), CliError> {
    let mut file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut file_manager);
    let parsed_files = parse_all(&file_manager);

    let pattern = if args.test_names.is_empty() {
        FunctionNameMatch::Anything
    } else if args.exact {
        FunctionNameMatch::Exact(args.test_names.clone())
    } else {
        FunctionNameMatch::Contains(args.test_names.clone())
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
    pattern: FunctionNameMatch,
    num_threads: usize,
    formatter: Box<dyn Formatter>,
}

impl<'a> TestRunner<'a> {
    fn run(&self) -> Result<(), CliError> {
        // First compile all packages and collect their tests
        let packages_tests = self.collect_packages_tests()?;

        if self.args.vector_tests {
            for (package_name, package_tests) in packages_tests {
                for test in package_tests {
                    println!("{} {}", package_name, test.name);
                }
            }
            return Ok(());
        }

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
                FunctionNameMatch::Exact(patterns) => {
                    if patterns.len() == 1 {
                        return Err(CliError::Generic(format!(
                            "Found 0 tests matching '{}'.",
                            patterns.first().unwrap()
                        )));
                    } else {
                        return Err(CliError::Generic(format!(
                            "Found 0 tests matching any of {}.",
                            patterns.join(", "),
                        )));
                    }
                }
                FunctionNameMatch::Contains(patterns) => {
                    if patterns.len() == 1 {
                        return Err(CliError::Generic(format!(
                            "Found 0 tests containing '{}'.",
                            patterns.first().unwrap()
                        )));
                    } else {
                        return Err(CliError::Generic(format!(
                            "Found 0 tests containing any of {}.",
                            patterns.join(", ")
                        )));
                    }
                }
                // If we are running all tests in a crate, having none is not an error
                FunctionNameMatch::Anything => {}
            };
        }

        if all_passed { Ok(()) } else { Err(CliError::Generic(String::new())) }
    }

    /// Process a chunk of tests sequentially and send the results to the main thread
    /// We need this functions, because first we process the standard tests, and then the fuzz tests.
    fn process_chunk_of_tests<I>(&self, iter_tests: &Mutex<I>, thread_sender: &Sender<TestResult>)
    where
        I: Iterator<Item = Test<'a>>,
    {
        loop {
            // Get next test to process from the iterator.
            let Some(test) = iter_tests.lock().unwrap().next() else {
                break;
            };

            self.formatter
                .test_start_async(&test.name, &test.package_name)
                .expect("Could not display test start");

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

            self.formatter
                .test_end_async(
                    &test_result,
                    self.file_manager,
                    self.args.show_output,
                    self.args.compile_options.deny_warnings,
                    self.args.compile_options.silence_warnings,
                )
                .expect("Could not display test start");

            if thread_sender.send(test_result).is_err() {
                break;
            }
        }
    }

    /// Runs all tests. Returns `true` if all tests passed, `false` otherwise.
    fn run_all_tests(
        &self,
        tests: Vec<Test<'a>>,
        test_count_per_package: &BTreeMap<String, usize>,
    ) -> bool {
        let mut all_passed = true;

        for (package_name, total_test_count) in test_count_per_package {
            self.formatter
                .package_start_async(package_name, *total_test_count)
                .expect("Could not display package start");
        }

        let (sender, receiver) = mpsc::channel();
        let (standard_tests_finished_sender, standard_tests_finished_receiver) = mpsc::channel();
        // Partition tests into standard and fuzz tests
        let (iter_tests_without_arguments, iter_tests_with_arguments): (
            Vec<Test<'a>>,
            Vec<Test<'a>>,
        ) = tests.into_iter().partition(|test| !test.has_arguments);

        // Calculate the actual number of threads needed based on test count.
        let num_threads = self.num_threads.min(iter_tests_without_arguments.len()).max(1);

        let iter_tests_without_arguments = &Mutex::new(iter_tests_without_arguments.into_iter());
        let iter_tests_with_arguments = &Mutex::new(iter_tests_with_arguments.into_iter());

        thread::scope(|scope| {
            // Start worker threads
            for _ in 0..num_threads {
                // Clone sender so it's dropped once the thread finishes
                let test_result_thread_sender = sender.clone();
                let standard_tests_finished_thread_sender = standard_tests_finished_sender.clone();
                thread::Builder::new()
                    // Specify a larger-than-default stack size to prevent overflowing stack in large programs.
                    // (the default is 2MB)
                    .stack_size(STACK_SIZE)
                    .spawn_scoped(scope, move || {
                        self.process_chunk_of_tests(
                            iter_tests_without_arguments,
                            &test_result_thread_sender,
                        );
                        // Signal that we've finished processing the standard tests in this thread
                        let _ = standard_tests_finished_thread_sender.send(());
                    })
                    .unwrap();
            }

            let test_result_thread_sender = sender.clone();
            thread::Builder::new()
                .stack_size(STACK_SIZE)
                .spawn_scoped(scope, move || {
                    let mut standard_tests_threads_finished = 0;
                    // Wait for at least half of the threads to finish processing the standard tests
                    while standard_tests_finished_receiver.recv().is_ok() {
                        standard_tests_threads_finished += 1;
                        if standard_tests_threads_finished >= max(1, num_threads / 2) {
                            break;
                        }
                    }

                    // Process fuzz tests sequentially
                    // Parallelism is handled by the fuzz tests themselves
                    self.process_chunk_of_tests(
                        iter_tests_with_arguments,
                        &test_result_thread_sender,
                    );
                })
                .unwrap();

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
                    .package_start_sync(package_name, total_test_count)
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
        // Calculate the actual number of threads needed based on package count.
        let num_threads = self.num_threads.min(self.workspace.members.len()).max(1);
        let iter = &Mutex::new(self.workspace.into_iter());

        thread::scope(|scope| {
            // Start worker threads
            for _ in 0..num_threads {
                // Clone sender so it's dropped once the thread finishes
                let thread_sender = sender.clone();
                thread::Builder::new()
                    // Specify a larger-than-default stack size to prevent overflowing stack in large programs.
                    // (the default is 2MB)
                    .stack_size(STACK_SIZE)
                    .spawn_scoped(scope, move || {
                        loop {
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

        if let Some(error) = error { Err(error) } else { Ok(package_tests) }
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
            .map(|(test_name, test_function)| {
                let test_name_copy = test_name.clone();
                let root_path = root_path.clone();
                let package_name_clone = package_name.clone();
                let package_name_clone2 = package_name.clone();
                let runner = Box::new(move || {
                    self.run_test::<S>(
                        package,
                        &test_name,
                        test_function.has_arguments,
                        foreign_call_resolver_url,
                        root_path,
                        package_name_clone.clone(),
                    )
                });
                Test {
                    name: test_name_copy,
                    package_name: package_name_clone2,
                    runner,
                    has_arguments: test_function.has_arguments,
                }
            })
            .collect();

        Ok(tests)
    }

    /// Compiles a single package and returns all of its test names
    fn get_tests_in_package(
        &'a self,
        package: &'a Package,
    ) -> Result<Vec<(String, TestFunction)>, CliError> {
        let (mut context, crate_id) =
            prepare_package(self.file_manager, self.parsed_files, package);
        check_crate_and_report_errors(&mut context, crate_id, &self.args.compile_options)?;

        Ok(context.get_all_test_functions_in_crate_matching(&crate_id, &self.pattern))
    }

    /// Runs a single test and returns its status together with whatever was printed to stdout
    /// during the test.
    fn run_test<S: BlackBoxFunctionSolver<FieldElement> + Default>(
        &'a self,
        package: &Package,
        fn_name: &str,
        has_arguments: bool,
        foreign_call_resolver_url: Option<&str>,
        root_path: Option<PathBuf>,
        package_name: String,
    ) -> (TestStatus, String) {
        if (self.args.no_fuzz && has_arguments) || (self.args.only_fuzz && !has_arguments) {
            return (TestStatus::Skipped, String::new());
        }

        // This is really hacky but we can't share `Context` or `S` across threads.
        // We then need to construct a separate copy for each test.

        let (mut context, crate_id) =
            prepare_package(self.file_manager, self.parsed_files, package);
        check_crate(&mut context, crate_id, &self.args.compile_options)
            .expect("Any errors should have occurred when collecting test functions");

        let pattern = FunctionNameMatch::Exact(vec![fn_name.to_string()]);
        let test_functions = context.get_all_test_functions_in_crate_matching(&crate_id, &pattern);
        let (_, test_function) = test_functions.first().expect("Test function should exist");

        if self.args.no_run {
            let status = match noirc_driver::compile_no_check(
                &mut context,
                &self.args.compile_options,
                test_function.id,
                None,
                false,
            ) {
                Ok(_) => TestStatus::Skipped,
                Err(err) => nargo::ops::test_status_program_compile_fail(err, test_function),
            };
            return (status, String::new());
        }

        let blackbox_solver = S::default();
        let mut output_buffer = Vec::new();

        let fuzz_config = FuzzConfig {
            folder_config: FuzzFolderConfig {
                corpus_dir: self.args.corpus_dir.clone(),
                minimized_corpus_dir: self.args.minimized_corpus_dir.clone(),
                fuzzing_failure_dir: self.args.fuzzing_failure_dir.clone(),
            },
            execution_config: FuzzExecutionConfig {
                num_threads: self.num_threads,
                timeout: self.args.fuzz_timeout,
                show_progress: self.args.fuzz_show_progress,
                max_executions: self.args.fuzz_max_executions,
            },
        };

        let test_status = nargo::ops::run_or_fuzz_test(
            &blackbox_solver,
            &mut context,
            test_function,
            &mut output_buffer,
            package_name.clone(),
            &self.args.compile_options,
            fuzz_config,
            |output, base| {
                DefaultForeignCallBuilder {
                    output,
                    enable_mocks: true,
                    resolver_url: foreign_call_resolver_url.map(|s| s.to_string()),
                    root_path: root_path.clone(),
                    package_name: Some(package_name.clone()),
                }
                .build_with_base(base)
            },
        );

        let output_string =
            String::from_utf8(output_buffer).expect("output buffer should contain valid utf8");

        (test_status, output_string)
    }

    /// Display the status of a single test
    fn display_test_result(
        &'a self,
        test_result: &'a TestResult,
        current_test_count: usize,
        total_test_count: usize,
    ) -> std::io::Result<()> {
        self.formatter.test_end_sync(
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
