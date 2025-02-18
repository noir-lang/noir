use std::{io::Write, path::PathBuf};

use acvm::{BlackBoxFunctionSolver, FieldElement};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;
use fm::FileManager;
use nargo::{
    insert_all_files_for_workspace_into_file_manager,
    ops::FuzzingRunStatus,
    package::{CrateName, Package},
    parse_all, prepare_package,
    workspace::Workspace,
    FuzzExecutionConfig, FuzzFolderConfig,
};
use nargo_toml::PackageSelection;
use noirc_abi::input_parser::{json::serialize_to_json, Format};
use noirc_driver::{check_crate, CompileOptions};
use noirc_frontend::hir::{FunctionNameMatch, ParsedFiles};
use rayon::prelude::{ParallelBridge, ParallelIterator};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::{cli::check_cmd::check_crate_and_report_errors, errors::CliError};

use super::{fs::inputs::write_inputs_to_file, LockType, PackageOptions, WorkspaceCommand};

/// Run the tests for this program
#[derive(Debug, Clone, Args)]
#[clap(visible_alias = "f")]
pub(crate) struct FuzzCommand {
    /// If given, only the fuzzing harnesses with names containing this string will be run
    fuzzing_harness_name: Option<String>,

    /// If given, load/store fuzzer corpus from this folder
    #[arg(long)]
    corpus_dir: Option<String>,

    /// If given, perform corpus minimization instead of fuzzing and store results in the given folder
    #[arg(long)]
    minimized_corpus_dir: Option<String>,

    /// If given, store the failing input in the given folder
    #[arg(long)]
    fuzzing_failure_dir: Option<String>,
    /// List all available harnesses that match the name
    #[clap(long)]
    list_all: bool,

    /// Display output of `println` statements
    #[arg(long)]
    show_output: bool,

    /// The number of threads to use for fuzzing
    #[arg(long, default_value = "1")]
    num_threads: usize,

    /// Only run harnesses that match exactly
    #[clap(long)]
    exact: bool,

    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    compile_options: CompileOptions,

    /// JSON RPC url to solve oracle calls
    #[clap(long)]
    oracle_resolver: Option<String>,

    /// Maximum time in seconds to spend fuzzing (default: no timeout)
    #[arg(long)]
    timeout: Option<u64>,
}
impl WorkspaceCommand for FuzzCommand {
    fn package_selection(&self) -> PackageSelection {
        self.package_options.package_selection()
    }
    fn lock_type(&self) -> LockType {
        // Reads the code to compile fuzzing harnesses in memory, but doesn't save artifacts.
        LockType::None
    }
}
pub(crate) fn run(args: FuzzCommand, workspace: Workspace) -> Result<(), CliError> {
    let mut file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut file_manager);
    let parsed_files = parse_all(&file_manager);

    let pattern = match &args.fuzzing_harness_name {
        Some(name) => {
            let names = vec![name.to_string()];
            if args.exact {
                FunctionNameMatch::Exact(names)
            } else {
                FunctionNameMatch::Contains(names)
            }
        }
        None => FunctionNameMatch::Anything,
    };

    if args.list_all {
        let pool = rayon::ThreadPoolBuilder::new().stack_size(4 * 1024 * 1024).build().unwrap();
        let all_harnesses_by_package: Vec<(CrateName, Vec<String>)> = pool
            .install(|| {
                workspace.into_iter().par_bridge().map(|package| {
                    let harnesses = list_harnesses(
                        &file_manager,
                        &parsed_files,
                        package,
                        &pattern,
                        &args.compile_options,
                    );
                    match harnesses {
                        Ok(harness_names) => Ok((package.name.clone(), harness_names)),
                        Err(cli_error) => Err(cli_error),
                    }
                })
            })
            .collect::<Result<_, _>>()?;
        let mut found_harness = false;
        for (crate_name, discovered_harnesses) in all_harnesses_by_package.iter() {
            if !discovered_harnesses.is_empty() {
                println!("Package {crate_name} contains fuzzing harnesses:");
                for harness in discovered_harnesses.iter() {
                    println!("\t{harness}");
                }
                found_harness = true;
            }
        }
        if !found_harness {
            println!("No fuzzing harnesses found");
        }
        return Ok(());
    }

    let fuzz_folder_config = FuzzFolderConfig {
        corpus_dir: args.corpus_dir,
        minimized_corpus_dir: args.minimized_corpus_dir,
        fuzzing_failure_dir: args.fuzzing_failure_dir,
    };
    let fuzz_execution_config =
        FuzzExecutionConfig { timeout: args.timeout.unwrap_or(0), num_threads: args.num_threads };

    let fuzzing_reports: Vec<Vec<(String, FuzzingRunStatus)>> = workspace
        .into_iter()
        .map(|package| {
            run_fuzzers::<Bn254BlackBoxSolver>(
                &file_manager,
                &parsed_files,
                package,
                &pattern,
                args.show_output,
                args.oracle_resolver.as_deref(),
                Some(workspace.root_dir.clone()),
                Some(package.name.to_string()),
                &args.compile_options,
                &fuzz_folder_config,
                &fuzz_execution_config,
            )
            .unwrap_or_else(|_| Vec::new())
        })
        .collect();

    let fuzzing_report: Vec<(String, FuzzingRunStatus)> =
        fuzzing_reports.into_iter().flatten().collect();

    if fuzzing_report.is_empty() {
        match &pattern {
            FunctionNameMatch::Exact(pattern) => {
                let single_pattern = pattern[0].clone();
                return Err(CliError::Generic(format!(
                    "Found 0 fuzzing_harnesses matching input '{single_pattern}'.",
                )));
            }
            FunctionNameMatch::Contains(pattern) => {
                let single_pattern = pattern[0].clone();
                return Err(CliError::Generic(format!(
                    "Found 0 fuzzing_harnesses containing '{single_pattern}'.",
                )));
            }
            // If we are running all tests in a crate, having none is not an error
            FunctionNameMatch::Anything => {}
        };
    }

    if fuzzing_report.iter().any(|(_, status)| status.failed()) {
        Err(CliError::Generic(String::new()))
    } else {
        Ok(())
    }
}

fn list_harnesses(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    fn_name: &FunctionNameMatch,
    compile_options: &CompileOptions,
) -> Result<Vec<String>, CliError> {
    let fuzzing_harnesses = get_fuzzing_harnesses_in_package(
        file_manager,
        parsed_files,
        package,
        fn_name,
        compile_options,
    )?;
    Ok(fuzzing_harnesses)
}

#[allow(clippy::too_many_arguments)]
fn run_fuzzers<S: BlackBoxFunctionSolver<FieldElement> + Default>(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    fn_name: &FunctionNameMatch,
    show_output: bool,
    foreign_call_resolver_url: Option<&str>,
    root_path: Option<PathBuf>,
    package_name: Option<String>,
    compile_options: &CompileOptions,
    fuzz_folder_config: &FuzzFolderConfig,
    fuzz_execution_config: &FuzzExecutionConfig,
) -> Result<Vec<(String, FuzzingRunStatus)>, CliError> {
    let fuzzing_harnesses = get_fuzzing_harnesses_in_package(
        file_manager,
        parsed_files,
        package,
        fn_name,
        compile_options,
    )?;

    let mut fuzzing_reports = Vec::new();
    for fuzzing_harness_name in fuzzing_harnesses.into_iter() {
        let status = run_fuzzing_harness::<S>(
            file_manager,
            parsed_files,
            package,
            &fuzzing_harness_name,
            show_output,
            foreign_call_resolver_url,
            root_path.clone(),
            package_name.clone(),
            compile_options,
            fuzz_folder_config,
            fuzz_execution_config,
        );
        fuzzing_reports.push((fuzzing_harness_name, status));
        // Display the latest report
        display_fuzzing_report_and_store(
            root_path.clone(),
            fuzz_folder_config.fuzzing_failure_dir.clone(),
            file_manager,
            package,
            compile_options,
            &fuzzing_reports[(&fuzzing_reports.len() - 1)..fuzzing_reports.len()],
        )?;
    }

    Ok(fuzzing_reports)
}

#[allow(clippy::too_many_arguments)]
fn run_fuzzing_harness<S: BlackBoxFunctionSolver<FieldElement> + Default>(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    fn_name: &str,
    show_output: bool,
    foreign_call_resolver_url: Option<&str>,
    root_path: Option<PathBuf>,
    package_name: Option<String>,
    compile_options: &CompileOptions,
    fuzz_folder_config: &FuzzFolderConfig,
    fuzz_execution_config: &FuzzExecutionConfig,
) -> FuzzingRunStatus {
    // This is really hacky but we can't share `Context` or `S` across threads.
    // We then need to construct a separate copy for each test.

    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);
    check_crate(&mut context, crate_id, compile_options)
        .expect("Any errors should have occurred when collecting fuzzing harnesses");

    let pattern = FunctionNameMatch::Exact(vec![fn_name.to_string()]);
    let fuzzing_harnesses =
        context.get_all_fuzzing_harnesses_in_crate_matching(&crate_id, &pattern);
    let (_, fuzzing_harness) = fuzzing_harnesses.first().expect("Fuzzing harness should exist");

    nargo::ops::run_fuzzing_harness::<S>(
        &mut context,
        fuzzing_harness,
        show_output,
        foreign_call_resolver_url,
        root_path,
        package_name,
        compile_options,
        &fuzz_folder_config,
        fuzz_execution_config,
    )
}

fn get_fuzzing_harnesses_in_package(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    fn_name: &FunctionNameMatch,
    options: &CompileOptions,
) -> Result<Vec<String>, CliError> {
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);
    check_crate_and_report_errors(&mut context, crate_id, options)?;

    Ok(context
        .get_all_fuzzing_harnesses_in_crate_matching(&crate_id, fn_name)
        .into_iter()
        .map(|(test_name, _)| test_name)
        .collect())
}

fn display_fuzzing_report_and_store(
    root_path: Option<PathBuf>,
    fuzzing_failure_folder: Option<String>,
    file_manager: &FileManager,
    package: &Package,
    compile_options: &CompileOptions,
    fuzzing_report: &[(String, FuzzingRunStatus)],
) -> Result<(), CliError> {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut writer = writer.lock();
    let fuzzing_failure_path =
        fuzzing_failure_folder.map(PathBuf::from).unwrap_or(root_path.clone().unwrap_or_default());
    if !fuzzing_failure_path.exists() {
        std::fs::create_dir_all(&fuzzing_failure_path)
            .expect("Failed to create fuzzing failure directory");
    }

    for (fuzzing_harness_name, test_status) in fuzzing_report {
        write!(writer, "[").expect("Failed to write to stderr");
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");

        write!(writer, "{}", package.name).expect("Failed to write to stderr");
        writer.reset().expect("Failed to reset writer");
        write!(writer, "] Executed fuzzing task on ").expect("Failed to write to stderr");
        writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).expect("Failed to set color");
        write!(writer, "{fuzzing_harness_name}").expect("Failed to write to stderr");
        writer.reset().expect("Failed to reset writer");
        write!(writer, "...").expect("Failed to write to stderr");
        writer.flush().expect("Failed to flush writer");

        match &test_status {
            FuzzingRunStatus::ExecutionPass { .. } => {
                writer
                    .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
                    .expect("Failed to set color");
                writeln!(writer, "ok").expect("Failed to write to stderr");
            }
            FuzzingRunStatus::MinimizationPass { .. } => {
                writer
                    .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
                    .expect("Failed to set color");
                writeln!(writer, "successfully minimized corpus")
                    .expect("Failed to write to stderr");
            }
            FuzzingRunStatus::CorpusFailure { message } => {
                writeln!(writer, "issue with corpus: ").expect("Failed to write to stderr");
                writer
                    .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                    .expect("Failed to set color");

                writeln!(writer, "{message}").expect("Failed to write to stderr");
                writer.reset().expect("Failed to reset writer");
            }
            FuzzingRunStatus::MinimizationFailure { message } => {
                writeln!(writer, "couldn't minimize corpus: ").expect("Failed to write to stderr");
                writer
                    .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                    .expect("Failed to set color");

                writeln!(writer, "{message}").expect("Failed to write to stderr");
                writer.reset().expect("Failed to reset writer");
            }
            FuzzingRunStatus::ForeignCallFailure { message } => {
                writeln!(writer, "issue with a foreign call: ").expect("Failed to write to stderr");
                writer
                    .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                    .expect("Failed to set color");

                writeln!(writer, "{message}").expect("Failed to write to stderr");
                writer.reset().expect("Failed to reset writer");
            }
            FuzzingRunStatus::ExecutionFailure { message, counterexample, error_diagnostic } => {
                write!(writer, "execution ").expect("Failed to write to stderr");
                writer
                    .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                    .expect("Failed to set color");
                write!(writer, "failed").expect("Failed to write to stderr");
                writer.reset().expect("Failed to reset writer");
                writeln!(writer, " with message:").expect("Failed to write to stderr");
                writer
                    .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))
                    .expect("Failed to set color");

                writeln!(writer, "{message}").expect("Failed to write to stderr");
                writer.reset().expect("Failed to reset writer");
                if let Some((input_map, abi)) = counterexample {
                    writeln!(writer, "Failing input: ",).expect("Failed to write to stderr");
                    writer
                        .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                        .expect("Failed to set color");
                    writeln!(
                        writer,
                        "{}",
                        serialize_to_json(input_map, abi)
                            .expect("Input map should be correctly serialized with this Abi")
                    )
                    .expect("Failed to write to stderr");
                    writer.reset().expect("Failed to reset writer");
                    let file_name = "Prover-failing-".to_owned()
                        + &package.name.to_string()
                        + "-"
                        + fuzzing_harness_name;
                    write_inputs_to_file(
                        fuzzing_failure_path.clone(),
                        &file_name,
                        Format::Toml,
                        abi,
                        input_map,
                    )
                    .expect("Couldn't write toml file");
                    writeln!(writer, "saved input to:").expect("Failed to write to stderr");
                    writer
                        .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))
                        .expect("Failed to set color");
                    let mut full_path_of_example =
                        PathBuf::from(fuzzing_failure_path.clone()).join(file_name);
                    full_path_of_example.set_extension(PathBuf::from("toml"));
                    writeln!(writer, "\"{}\"", full_path_of_example.to_str().unwrap())
                        .expect("Failed to write to stderr");
                    writer.reset().expect("Failed to reset writer");
                }
                if let Some(diag) = error_diagnostic {
                    noirc_errors::reporter::report_all(
                        file_manager.as_file_map(),
                        &[diag.clone()],
                        compile_options.deny_warnings,
                        compile_options.silence_warnings,
                    );
                }
            }
            FuzzingRunStatus::CompileError(err) => {
                noirc_errors::reporter::report_all(
                    file_manager.as_file_map(),
                    &[err.clone()],
                    compile_options.deny_warnings,
                    compile_options.silence_warnings,
                );
            }
        }
        writer.reset().expect("Failed to reset writer");
        writer.flush().expect("Failed to flush writer");
    }

    write!(writer, "[{}] ", package.name).expect("Failed to write to stderr");

    let count_all = fuzzing_report.len();
    let count_failed = fuzzing_report.iter().filter(|(_, status)| status.failed()).count();
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
