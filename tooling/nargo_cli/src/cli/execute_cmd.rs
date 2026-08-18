use std::path::PathBuf;

use clap::Args;

use nargo::constants::PROVER_INPUT_FILE;
use nargo::foreign_calls::OracleResolverUrl;
use nargo::ops::report_errors;
use nargo::prepare_package;
use nargo::workspace::Workspace;
use nargo_toml::PackageSelection;
use noirc_driver::{CompileOptions, compile_main, link_to_debug_crate};

use super::compile_cmd::{compile_workspace_full, parse_workspace};
use super::{LockType, PackageOptions, WorkspaceCommand};
use crate::cli::execute_cmd::interpret::run_comptime;
use crate::errors::CliError;

mod interpret;

/// Executes a circuit to calculate its return value
#[derive(Debug, Clone, Args)]
#[clap(visible_alias = "e")]
pub(crate) struct ExecuteCommand {
    /// Write the execution witness to named file
    ///
    /// Defaults to the name of the package being executed.
    witness_name: Option<String>,

    /// The name of the toml file which contains the inputs for the prover
    #[clap(long, short, default_value = PROVER_INPUT_FILE)]
    prover_name: String,

    /// Optionally overwrite the `return` entry in the prover file.
    #[clap(long, default_value_t = false)]
    pub overwrite_return: bool,

    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    compile_options: CompileOptions,

    /// JSON RPC url to solve oracle calls
    #[clap(long, conflicts_with = "oracle_file")]
    oracle_resolver: Option<OracleResolverUrl>,

    /// Path to the oracle transcript.
    #[clap(long, conflicts_with = "oracle_resolver")]
    oracle_file: Option<PathBuf>,

    /// Force comptime execution
    #[arg(long, hide = true)]
    force_comptime: bool,

    /// Count the number of arrays that are copied in an unconstrained context for performance
    /// debugging.
    #[arg(long)]
    count_array_copies: bool,
}

impl WorkspaceCommand for ExecuteCommand {
    fn package_selection(&self) -> PackageSelection {
        self.package_options.package_selection()
    }

    fn lock_type(&self) -> LockType {
        // Compiles artifacts.
        LockType::Exclusive
    }
}

pub(crate) fn run(args: ExecuteCommand, workspace: Workspace) -> Result<(), CliError> {
    if args.force_comptime {
        return run_comptime(args, workspace);
    }

    if args.count_array_copies {
        return execute_without_artifacts(args, workspace);
    }

    // Compile the full workspace in order to generate any build artifacts.
    let debug_compile_stdin = None;
    compile_workspace_full(&workspace, &args.compile_options, debug_compile_stdin)?;

    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());
    for package in binary_packages {
        let program_artifact_path = workspace.package_build_path(package);
        let prover_file = package.root_dir.join(&args.prover_name).with_extension("toml");

        let cmd = noir_artifact_cli::commands::execute_cmd::ExecuteCommand {
            artifact_path: program_artifact_path,
            prover_file,
            overwrite_return: args.overwrite_return,
            output_dir: Some(workspace.target_directory_path()),
            witness_name: Some(
                args.witness_name.clone().unwrap_or_else(|| package.name.to_string()),
            ),
            contract_fn: None,
            oracle_file: args.oracle_file.clone(),
            oracle_resolver: args.oracle_resolver.clone(),
            oracle_root_dir: Some(workspace.root_dir.clone()),
            oracle_package_name: Some(package.name.to_string()),
        };

        noir_artifact_cli::commands::execute_cmd::run(cmd)?;
    }
    Ok(())
}

/// Compile and execute each binary package in memory, without reading or writing any
/// compilation artifact. Used for `--count-array-copies`, whose Brillig instrumentation must
/// not be persisted to (or served from) the artifact cache.
fn execute_without_artifacts(args: ExecuteCommand, workspace: Workspace) -> Result<(), CliError> {
    let (file_manager, parsed_files) = parse_workspace(&workspace, None);

    for package in workspace.into_iter().filter(|package| package.is_binary()) {
        let (mut context, crate_id) = prepare_package(&file_manager, &parsed_files, package);
        link_to_debug_crate(&mut context, crate_id);
        context.package_build_path = workspace.package_build_path(package);
        context.count_array_copies = true;

        // Passing no cached program ignores any previously persisted, un-instrumented artifact.
        let compilation_result = compile_main(&mut context, crate_id, &args.compile_options, None);
        let program = report_errors(
            compilation_result,
            &file_manager,
            &parsed_files,
            args.compile_options.deny_warnings,
            args.compile_options.silence_warnings,
        )?;

        let prover_file = package.root_dir.join(&args.prover_name).with_extension("toml");
        let circuit_name = package.name.to_string();
        let witness_name = args.witness_name.clone().unwrap_or_else(|| circuit_name.clone());
        // Save the witness as a normal `execute` would, but never the program artifact.
        noir_artifact_cli::commands::execute_cmd::execute_program(
            &program,
            &circuit_name,
            &noir_artifact_cli::commands::execute_cmd::ExecuteProgramArgs {
                prover_file: &prover_file,
                output_dir: Some(&workspace.target_directory_path()),
                witness_name: Some(&witness_name),
                overwrite_return: args.overwrite_return,
                oracle_file: args.oracle_file.as_deref(),
                oracle_resolver: args.oracle_resolver.as_ref(),
            },
        )?;
    }
    Ok(())
}
