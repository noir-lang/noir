//! `nargo test`: find the tests in each package, run them on worker threads and show the results.
//!
//! The flow, top to bottom in this file:
//! 1. [`run`] builds a [`TestRunner`] from the command-line arguments.
//! 2. [`TestRunner::collect_tests`] elaborates every package and decides, once per test, how it
//!    runs ([`RunMode`]), or that it is skipped.
//! 3. [`TestRunner::run_tests`] hands the tests to worker threads. Each worker runs tests one at a
//!    time ([`TestRunner::run_worker`]) and sends every result to the main thread.
//! 4. [`TestRunner::display_results`] shows the results package by package as they arrive.

use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, VecDeque},
    fmt::Display,
    io,
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    rc::Rc,
    sync::{
        Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;
use fm::FileManager;
use formatters::{DisplayOptions, Formatter, JsonFormatter, PrettyFormatter, TerseFormatter};
use nargo::{
    FuzzExecutionConfig, FuzzFolderConfig,
    foreign_calls::{DefaultForeignCallBuilder, OracleResolverUrl},
    insert_all_files_for_workspace_into_file_manager,
    ops::{FuzzConfig, TestStatus, report_errors},
    package::Package,
    parse_all, prepare_package,
    workspace::Workspace,
};
use nargo_toml::PackageSelection;
use noirc_driver::{CompilationResult, CompileOptions, check_crate};
use noirc_frontend::graph::CrateId;
use noirc_frontend::hir::{
    Context, FunctionNameMatch, ParsedFiles, comptime::EvaluationTracker, def_map::TestFunction,
};

use crate::errors::CliError;

use super::{LockType, PackageOptions, WorkspaceCommand, parse_and_normalize_path};

mod coverage;
pub(crate) mod formatters;

/// Fully qualified test name.
type TestName = String;
type PackageName = String;

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
    list_tests: bool,

    /// Only compile the tests, without running them.
    #[clap(long)]
    no_run: bool,

    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    compile_options: CompileOptions,

    /// JSON RPC url to solve oracle calls
    #[clap(long)]
    oracle_resolver: Option<OracleResolverUrl>,

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

    /// Elaborate the package again for every test rather than sharing one elaboration per thread
    ///
    /// Sharing is a large speedup on packages with many tests, but it means a test compiles
    /// against a context that earlier tests on the same thread have already compiled against.
    /// Use this to check whether a surprising result depends on what ran before it.
    #[clap(long)]
    no_context_reuse: bool,

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

    /// Force comptime execution
    ///
    /// This only works with tests that don't have arguments and don't call Oracles.
    #[arg(long, hide = true)]
    force_comptime: bool,

    /// Produce a coverage report.
    ///
    /// Writes coverage data to the workspace target directory into
    /// `target/coverage/<package-name>/lcov.info` or `target/coverage/lcov.info` files,
    /// depending on whether we are dealing with a workspace.
    #[arg(long)]
    coverage: bool,

    /// Override the directory where coverage files are written.
    ///
    /// If not set, defaults to the workspace target directory.
    #[arg(long, value_parser = parse_and_normalize_path)]
    coverage_dir: Option<PathBuf>,
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
    fn formatter<'a>(&self, options: DisplayOptions<'a>) -> Box<dyn Formatter + 'a> {
        match self {
            Format::Pretty => Box::new(PrettyFormatter::new(options)),
            Format::Terse => Box::new(TerseFormatter::new(options)),
            Format::Json => Box::new(JsonFormatter::new(options)),
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

    let display_options = DisplayOptions {
        file_manager: &file_manager,
        parsed_files: &parsed_files,
        show_output: args.show_output,
        deny_warnings: args.compile_options.deny_warnings,
        silence_warnings: args.compile_options.silence_warnings,
    };
    let format = args.format.unwrap_or(if args.quiet { Format::Terse } else { Format::Pretty });

    let runner = TestRunner {
        file_manager: &file_manager,
        parsed_files: &parsed_files,
        workspace,
        args: &args,
        pattern,
        formatter: format.formatter(display_options),
    };
    runner.run()
}

struct TestRunner<'a> {
    file_manager: &'a FileManager,
    parsed_files: &'a ParsedFiles,
    workspace: Workspace,
    args: &'a TestCommand,
    /// Which tests to collect, from the test names given on the command line.
    pattern: FunctionNameMatch,
    formatter: Box<dyn Formatter + 'a>,
}

impl<'a> TestRunner<'a> {
    fn run(&self) -> Result<(), CliError> {
        let packages = self.collect_tests()?;

        if self.args.list_tests {
            for (package_name, package) in &packages {
                for test in &package.tests {
                    noirc_errors::println_to_stdout!("{} {}", package_name, test.name);
                }
            }
            return Ok(());
        }

        let found_tests = packages.values().any(|package| !package.tests.is_empty());
        let all_passed = self.run_tests(packages).map_err(output_error)?;

        if !found_tests && let Some(error) = no_tests_found_error(&self.pattern) {
            return Err(error);
        }
        if all_passed { Ok(()) } else { Err(CliError::Generic(String::new())) }
    }

    // --- Collecting tests ---

    /// Elaborates every package in parallel and returns the tests matching [`Self::pattern`].
    fn collect_tests(&'a self) -> Result<BTreeMap<PackageName, PackageTests<'a>>, CliError> {
        let num_threads = self.args.test_threads.min(self.workspace.members.len()).max(1);
        let packages = &Mutex::new(self.workspace.into_iter());
        let (sender, receiver) = mpsc::channel();

        thread::scope(|scope| {
            for _ in 0..num_threads {
                let sender = sender.clone();
                spawn_worker(scope, move || {
                    loop {
                        let Some(package) = packages.lock().unwrap().next() else {
                            break;
                        };
                        if sender.send((package, self.collect_package_tests(package))).is_err() {
                            break;
                        }
                    }
                });
            }
        });
        drop(sender);

        let mut collected = BTreeMap::new();
        let mut error = None;
        for (package, result) in receiver {
            match result {
                Ok(tests) => {
                    collected.insert(package.name.to_string(), tests);
                }
                Err(err) => error = Some(err),
            }
        }
        match error {
            Some(error) => Err(error),
            None => Ok(collected),
        }
    }

    /// Elaborates `package`, reporting its errors and warnings, and returns its tests.
    fn collect_package_tests(&'a self, package: &'a Package) -> Result<PackageTests<'a>, CliError> {
        let (context, crate_id, result) = self.elaborate(package);
        report_errors(
            result,
            &context.file_manager,
            &context.parsed_files,
            self.args.compile_options.deny_warnings,
            self.args.compile_options.silence_warnings,
        )?;

        let tests = context
            .get_all_test_functions_in_crate_matching(&crate_id, &self.pattern)
            .into_iter()
            .map(|(name, function)| Test {
                name,
                package,
                package_name: package.name.to_string(),
                has_arguments: function.has_arguments,
                run_mode: self.run_mode(function.has_arguments),
            })
            .collect();

        // The baseline needs the elaborated context, which is only at hand here.
        let coverage_baseline =
            self.args.coverage.then(|| coverage::baseline_in_package(&context, crate_id));

        Ok(PackageTests { tests, coverage_baseline })
    }

    /// How a test runs under the command-line flags, or `None` if they exclude it.
    fn run_mode(&self, has_arguments: bool) -> Option<RunMode> {
        let args = self.args;
        let excluded =
            if has_arguments { args.no_fuzz || args.force_comptime } else { args.only_fuzz };

        if excluded {
            None
        } else if args.no_run {
            Some(RunMode::CompileOnly)
        } else if args.force_comptime || (args.coverage && !has_arguments) {
            Some(RunMode::Interpret)
        } else {
            Some(RunMode::Execute)
        }
    }

    /// Elaborates `package`, returning its context, its root crate and the errors and warnings
    /// found.
    fn elaborate(
        &'a self,
        package: &'a Package,
    ) -> (Context<'a, 'a>, CrateId, CompilationResult<()>) {
        let (mut context, crate_id) =
            prepare_package(self.file_manager, self.parsed_files, package);

        if self.args.coverage {
            // Set the tracker before elaboration so comptime blocks executed during
            // check_crate are captured. We use all file IDs known at this point since
            // def_maps isn't populated yet; after check_crate we narrow to crate files.
            let all_files = context.file_manager.as_file_map().all_file_ids().copied().collect();
            context.evaluation_tracker = Some(EvaluationTracker::new(all_files));
        }

        let result = check_crate(&mut context, crate_id, &self.args.compile_options);

        if let Some(evaluation_tracker) = context.evaluation_tracker.as_mut() {
            let crate_files = context.def_maps[&crate_id].file_ids();
            evaluation_tracker.restrict_to_files(&crate_files);
        }

        (context, crate_id, result)
    }

    // --- Running tests ---

    /// Runs every test on worker threads and shows the results package by package.
    /// Returns whether all tests passed.
    fn run_tests(&'a self, packages: BTreeMap<PackageName, PackageTests<'a>>) -> io::Result<bool> {
        let mut tests = Vec::new();
        let mut reports = BTreeMap::new();
        for (package_name, package) in packages {
            let test_count = package.tests.len();
            self.formatter.package_start_async(&package_name, test_count)?;
            reports.insert(
                package_name,
                PackageReport { test_count, coverage: package.coverage_baseline },
            );
            tests.extend(package.tests);
        }

        // A fuzz test spreads itself over its own threads, so fuzz tests run one at a time on a
        // single worker, which starts once half of the regular workers are done.
        let (regular_tests, fuzz_tests): (Vec<_>, Vec<_>) =
            tests.into_iter().partition(|test| !test.has_arguments);
        let num_threads = self.args.test_threads.min(regular_tests.len()).max(1);
        let regular_tests = &Mutex::new(regular_tests.into_iter());
        let fuzz_tests = &Mutex::new(fuzz_tests.into_iter());

        let (result_sender, result_receiver) = mpsc::channel();
        let (regular_done_sender, regular_done_receiver) = mpsc::channel();

        thread::scope(|scope| {
            let mut workers = Vec::with_capacity(num_threads + 1);

            for _ in 0..num_threads {
                let result_sender = result_sender.clone();
                let regular_done_sender = regular_done_sender.clone();
                workers.push(spawn_worker(scope, move || {
                    let result = self.run_worker(regular_tests, &result_sender);
                    let _ = regular_done_sender.send(());
                    result
                }));
            }
            drop(regular_done_sender);

            workers.push(spawn_worker(scope, move || {
                let _ = regular_done_receiver.iter().take((num_threads / 2).max(1)).count();
                self.run_worker(fuzz_tests, &result_sender)
            }));

            // `display_results` owns the receiver, so if it fails to write a result the receiver
            // is dropped, the workers' next send fails and they stop picking up tests.
            let all_passed = self.display_results(result_receiver, reports)?;

            // A worker that fails to write stops early, which leaves its package short of
            // results; its error explains why.
            for worker in workers {
                worker.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic))?;
            }

            Ok(all_passed)
        })
    }

    /// Takes tests from `tests` until none are left, runs each one and sends its result to
    /// [`Self::display_results`].
    fn run_worker(
        &'a self,
        tests: &Mutex<impl Iterator<Item = Test<'a>>>,
        results: &Sender<FinishedTest>,
    ) -> io::Result<()> {
        let mut cached = None;

        loop {
            // `let ... else` releases the lock before the test runs, so other workers can take
            // tests in the meantime.
            let Some(test) = tests.lock().unwrap().next() else {
                break;
            };

            self.formatter.test_start_async(&test.name, &test.package_name)?;
            let started = Instant::now();

            let outcome = match test.run_mode {
                // A skipped test needs no context, so the fuzzing flags never elaborate a package
                // whose tests they all skip.
                None => TestOutcome::status_only(TestStatus::Skipped),
                Some(run_mode) => self.run_test(&mut cached, &test, run_mode),
            };

            let result = TestResult {
                name: test.name,
                package_name: test.package_name,
                status: outcome.status,
                output: outcome.output,
                time_to_run: started.elapsed(),
            };
            self.formatter.test_end_async(&result)?;

            if results.send(FinishedTest { result, coverage: outcome.coverage }).is_err() {
                break;
            }
        }
        Ok(())
    }

    /// Runs `test` against the worker's cached context.
    ///
    /// A panic, even one raised while elaborating, fails only this test. Whether a test passed or
    /// even compiled leaves the context fit for the next test, but a test that did not finish gives
    /// up its context however far it got. `--no-context-reuse` gives it up after every test.
    fn run_test(
        &'a self,
        cached: &mut Option<CachedContext<'a>>,
        test: &Test<'a>,
        run_mode: RunMode,
    ) -> TestOutcome {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let cached = self.cached_context_for(cached, test);
            self.run_test_in_context(cached, test, run_mode)
        }));

        if self.args.no_context_reuse || outcome.is_err() {
            *cached = None;
        }

        outcome.unwrap_or_else(|panic| {
            // `panic!("...")` carries a `&str`, which covers the common case.
            let message = panic.downcast_ref::<&str>().copied();
            let message = message.unwrap_or("An unexpected error happened").to_string();
            TestOutcome::status_only(TestStatus::Fail { message, error_diagnostic: None })
        })
    }

    /// Return the context to compile `test` against, elaborating `test`'s package into `cached`
    /// unless it already holds an elaboration of that same package.
    ///
    /// A workspace hands its packages to the worker threads through one shared iterator, so
    /// consecutive tests on a thread are not necessarily from the same package.
    fn cached_context_for<'b>(
        &'a self,
        cached: &'b mut Option<CachedContext<'a>>,
        test: &Test<'a>,
    ) -> &'b mut CachedContext<'a> {
        if !cached.as_ref().is_some_and(|cached| std::ptr::eq(cached.package, test.package)) {
            let (context, crate_id, result) = self.elaborate(test.package);
            result.expect("Any errors should have occurred when collecting test functions");
            let tracker_after_elaboration = context.evaluation_tracker.clone();
            *cached = Some(CachedContext {
                package: test.package,
                context,
                crate_id,
                tracker_after_elaboration,
            });
        }
        cached.as_mut().expect("just populated")
    }

    fn run_test_in_context(
        &'a self,
        cached: &mut CachedContext<'a>,
        test: &Test<'a>,
        run_mode: RunMode,
    ) -> TestOutcome {
        let CachedContext { context, crate_id, tracker_after_elaboration, .. } = cached;

        let pattern = FunctionNameMatch::Exact(vec![test.name.to_string()]);
        let test_functions = context.get_all_test_functions_in_crate_matching(crate_id, &pattern);
        let (_, test_function) = test_functions.first().expect("Test function should exist");

        match run_mode {
            RunMode::CompileOnly => {
                TestOutcome::status_only(self.compile_only(context, test_function))
            }
            RunMode::Interpret => {
                self.interpret(context, tracker_after_elaboration, test, test_function)
            }
            RunMode::Execute => self.execute(context, test, test_function),
        }
    }

    /// Compiles `test_function` without running it. A test that compiles is reported as skipped.
    fn compile_only(&self, context: &Context, test_function: &TestFunction) -> TestStatus {
        match noirc_driver::compile_no_check(
            context,
            &self.args.compile_options,
            test_function.id,
            None,
            false,
        ) {
            Ok(_) => TestStatus::Skipped,
            Err(err) => nargo::ops::test_status_program_compile_fail(err, test_function),
        }
    }

    /// Runs `test_function` in the comptime interpreter, collecting its coverage if requested.
    fn interpret(
        &'a self,
        context: &mut Context<'a, 'a>,
        tracker_after_elaboration: &Option<EvaluationTracker>,
        test: &Test<'a>,
        test_function: &TestFunction,
    ) -> TestOutcome {
        let output = Rc::new(RefCell::new(Vec::new()));
        context.set_comptime_printing(output.clone());

        let result = context.interpret_function(test_function.id, Vec::new());
        let status = nargo::ops::test_status_comptime_interpret_result(result, test_function);

        context.interpreter_output = None;
        let output = Rc::try_unwrap(output).expect("context no longer has it");
        let output = String::from_utf8(output.into_inner()).expect("not UTF-8");

        let coverage = context.evaluation_tracker.take().map(|tracker| {
            coverage::tracker_to_report(&tracker, test_function.id, test.name.as_str(), context)
        });

        // Restore the post-elaboration tracker so the next test starts with a fresh copy
        // rather than forcing a full re-elaboration.
        context.evaluation_tracker = tracker_after_elaboration.clone();
        TestOutcome { status, output, coverage }
    }

    /// Compiles and executes `test_function`, fuzzing it if it has arguments.
    fn execute(
        &'a self,
        context: &Context<'a, 'a>,
        test: &Test<'a>,
        test_function: &TestFunction,
    ) -> TestOutcome {
        let mut output_buffer = Vec::new();

        let status = nargo::ops::run_or_fuzz_test(
            &Bn254BlackBoxSolver,
            context,
            test_function,
            &mut output_buffer,
            test.package_name.clone(),
            &self.args.compile_options,
            self.fuzz_config(),
            |output, base| {
                DefaultForeignCallBuilder {
                    output,
                    enable_mocks: true,
                    resolver_url: self
                        .args
                        .oracle_resolver
                        .as_ref()
                        .map(|url| url.as_str().to_string()),
                    root_path: Some(self.workspace.root_dir.clone()),
                    package_name: Some(test.package_name.clone()),
                }
                .build_with_base(base)
            },
        );

        let output =
            String::from_utf8(output_buffer).expect("output buffer should contain valid utf8");
        TestOutcome { status, output, coverage: None }
    }

    fn fuzz_config(&self) -> FuzzConfig {
        FuzzConfig {
            folder_config: FuzzFolderConfig {
                corpus_dir: self.args.corpus_dir.clone(),
                minimized_corpus_dir: self.args.minimized_corpus_dir.clone(),
                fuzzing_failure_dir: self.args.fuzzing_failure_dir.clone(),
            },
            execution_config: FuzzExecutionConfig {
                num_threads: self.args.test_threads,
                timeout: self.args.fuzz_timeout,
                show_progress: self.args.fuzz_show_progress,
                max_executions: self.args.fuzz_max_executions,
            },
        }
    }

    // --- Displaying results ---

    /// Shows results one package at a time, in package order, though the workers finish tests in
    /// any order. Writes each package's coverage report once its tests are shown.
    /// Returns whether all tests passed.
    fn display_results(
        &self,
        results: Receiver<FinishedTest>,
        packages: BTreeMap<PackageName, PackageReport>,
    ) -> io::Result<bool> {
        let mut all_passed = true;
        // Results that arrived before it was their package's turn.
        let mut held_back = HashMap::new();

        for (package_name, PackageReport { test_count, mut coverage }) in packages {
            self.formatter.package_start_sync(&package_name, test_count)?;

            let mut shown = Vec::with_capacity(test_count);
            while shown.len() < test_count {
                let Some(finished) = next_result_for(&package_name, &results, &mut held_back)
                else {
                    break;
                };

                all_passed &= !finished.result.status.failed();
                if let (Some(coverage), Some(test_coverage)) =
                    (coverage.as_mut(), finished.coverage)
                {
                    coverage.merge_lossy(test_coverage);
                }

                self.formatter.test_end_sync(&finished.result, shown.len() + 1, test_count)?;
                shown.push(finished.result);
            }

            self.formatter.package_end(&package_name, &shown)?;

            if let Some(coverage) = coverage {
                let lcov_path = coverage::package_lcov_path(
                    &self.workspace,
                    &package_name,
                    self.args.coverage_dir.as_deref(),
                );
                coverage::write_package_coverage(coverage, &lcov_path);
            }
        }

        Ok(all_passed)
    }
}

/// The next result for `package_name`: one held back earlier, or else the next one to arrive.
/// Results for other packages that arrive first are held back until their package's turn.
/// Returns `None` once every worker has stopped.
fn next_result_for(
    package_name: &str,
    results: &Receiver<FinishedTest>,
    held_back: &mut HashMap<PackageName, VecDeque<FinishedTest>>,
) -> Option<FinishedTest> {
    if let Some(finished) = held_back.get_mut(package_name).and_then(VecDeque::pop_front) {
        return Some(finished);
    }
    loop {
        let finished = results.recv().ok()?;
        if finished.result.package_name == package_name {
            return Some(finished);
        }
        held_back.entry(finished.result.package_name.clone()).or_default().push_back(finished);
    }
}

/// Worker threads get a larger-than-default stack (the default is 2MB) so that compiling large
/// programs doesn't overflow it.
const STACK_SIZE: usize = 4 * 1024 * 1024;

fn spawn_worker<'scope, T: Send + 'scope>(
    scope: &'scope thread::Scope<'scope, '_>,
    work: impl FnOnce() -> T + Send + 'scope,
) -> thread::ScopedJoinHandle<'scope, T> {
    thread::Builder::new().stack_size(STACK_SIZE).spawn_scoped(scope, work).unwrap()
}

/// The command's error when writing the test results fails.
fn output_error(error: io::Error) -> CliError {
    // Whoever was reading stdout has gone away, as in `nargo test | head`, so there is nobody to
    // show an error to. The run still did not complete, so it must not report success.
    if error.kind() == io::ErrorKind::BrokenPipe {
        CliError::Generic(String::new())
    } else {
        CliError::TestOutput(error)
    }
}

/// The error for a run that found no tests. Running every test of a crate that has none is not an
/// error, so there is none when no test names were given.
fn no_tests_found_error(pattern: &FunctionNameMatch) -> Option<CliError> {
    let (relation, names) = match pattern {
        FunctionNameMatch::Anything => return None,
        FunctionNameMatch::Exact(names) => ("matching", names),
        FunctionNameMatch::Contains(names) => ("containing", names),
    };
    let message = match names.as_slice() {
        [name] => format!("Found 0 tests {relation} '{name}'."),
        _ => format!("Found 0 tests {relation} any of {}.", names.join(", ")),
    };
    Some(CliError::Generic(message))
}

// --- Data passed between the steps ---

/// The tests collected from one package.
struct PackageTests<'a> {
    tests: Vec<Test<'a>>,
    /// The functions and lines the package's tests can cover, under `--coverage`.
    coverage_baseline: Option<lcov::Report>,
}

/// A test found in a package, with how it is going to run.
struct Test<'a> {
    name: TestName,
    package: &'a Package,
    package_name: PackageName,
    /// A test with arguments is a fuzz test.
    has_arguments: bool,
    /// `None` when the fuzzing flags exclude the test: it is reported as skipped and never compiled.
    run_mode: Option<RunMode>,
}

/// How a test that is not skipped runs, decided by [`TestRunner::run_mode`].
#[derive(Clone, Copy)]
enum RunMode {
    /// `--no-run`: compile the test without running it.
    CompileOnly,
    /// Run the test in the comptime interpreter.
    Interpret,
    /// Compile and execute the test, fuzzing it if it has arguments.
    Execute,
}

/// What running one test produced.
struct TestOutcome {
    status: TestStatus,
    /// What the test printed.
    output: String,
    /// The lines this test covered, for interpreted tests under `--coverage`.
    coverage: Option<lcov::Report>,
}

impl TestOutcome {
    fn status_only(status: TestStatus) -> Self {
        TestOutcome { status, output: String::new(), coverage: None }
    }
}

/// A test result on its way from a worker thread to [`TestRunner::display_results`].
struct FinishedTest {
    result: TestResult,
    coverage: Option<lcov::Report>,
}

/// What [`TestRunner::display_results`] needs to know about a package.
struct PackageReport {
    test_count: usize,
    /// The package's coverage report, merged with each test's coverage as its result is shown.
    coverage: Option<lcov::Report>,
}

pub(crate) struct TestResult {
    name: TestName,
    package_name: PackageName,
    status: TestStatus,
    output: String,
    time_to_run: Duration,
}

impl TestResult {
    pub(crate) fn new(
        name: TestName,
        package_name: PackageName,
        status: TestStatus,
        output: String,
        time_to_run: Duration,
    ) -> Self {
        TestResult { name, package_name, status, output, time_to_run }
    }
}

/// An elaborated [`Context`] kept alive across the tests a worker thread runs.
///
/// Elaborating a package is the single most expensive part of `nargo test` on a large program and
/// produces the same result for every test in that package, so a worker holds onto the context it
/// built and reuses it for the next test from the same package.
///
/// Reuse rests on monomorphization not changing what an elaborated context already holds: it
/// resolves generics through a substitution of its own rather than by binding the context's type
/// variables. `noirc_frontend::monomorphization::context_reuse_tests` asserts that a test compiles
/// to the same program whatever was compiled against the context before it. A context is dropped
/// when a test unwinds, and `--no-context-reuse` turns sharing off for a whole run.
struct CachedContext<'a> {
    package: &'a Package,
    context: Context<'a, 'a>,
    crate_id: CrateId,
    /// Post-elaboration snapshot of the evaluation tracker, cloned back into
    /// the context before each coverage test so the tracker is not consumed.
    tracker_after_elaboration: Option<EvaluationTracker>,
}
