use std::path::Path;

use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;

use nargo::constants::PROVER_INPUT_FILE;
use nargo::foreign_calls::DefaultForeignCallBuilder;
use nargo::workspace::Workspace;
use nargo::PrintOutput;
use nargo_toml::PackageSelection;
use noir_artifact_cli::fs::artifact::read_program_from_file;
use noirc_driver::{CompileOptions, CompiledProgram};

use super::compile_cmd::compile_workspace_full;
use super::{LockType, PackageOptions, WorkspaceCommand};
use crate::errors::CliError;

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

    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    compile_options: CompileOptions,

    /// JSON RPC url to solve oracle calls
    #[clap(long)]
    oracle_resolver: Option<String>,
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
    // Compile the full workspace in order to generate any build artifacts.
    compile_workspace_full(&workspace, &args.compile_options)?;

    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());
    for package in binary_packages {
        let program_artifact_path = workspace.package_build_path(package);
        let program: CompiledProgram = read_program_from_file(&program_artifact_path)?.into();
        let prover_file = package.root_dir.join(&args.prover_name).with_extension("toml");

        execute_program_and_save_witness(
            &args,
            &workspace,
            package.name.to_string(),
            program,
            &prover_file,
        )?;
    }
    Ok(())
}

fn execute_program_and_save_witness(
    args: &ExecuteCommand,
    workspace: &Workspace,
    package_name: String,
    compiled_program: CompiledProgram,
    prover_file: &Path,
) -> Result<(), CliError> {
    let blackbox_solver = Bn254BlackBoxSolver(args.compile_options.pedantic_solving);

    let mut foreign_call_executor = DefaultForeignCallBuilder {
        output: PrintOutput::Stdout,
        enable_mocks: false,
        resolver_url: args.oracle_resolver.clone(),
        root_path: Some(workspace.root_dir.to_path_buf()),
        package_name: Some(package_name.clone()),
    }
    .build();

    match noir_artifact_cli::execution::execute(
        &compiled_program,
        &blackbox_solver,
        &mut foreign_call_executor,
        prover_file,
    ) {
        Ok(results) => {
            let witness_name = args.witness_name.as_ref().unwrap_or(&package_name);

            noir_artifact_cli::execution::save_and_check_witness(
                &compiled_program,
                results,
                &package_name,
                Some(&workspace.target_directory_path()),
                Some(witness_name),
            )?;
        }
        Err(noir_artifact_cli::errors::CliError::CircuitExecutionError(err)) => {
            noir_artifact_cli::execution::show_diagnostic(compiled_program, err);
        }
        Err(e) => {
            return Err(e.into());
        }
    }
    Ok(())
}
