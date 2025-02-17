use clap::Args;

use nargo::constants::PROVER_INPUT_FILE;
use nargo::workspace::Workspace;
use nargo_toml::PackageSelection;
use noirc_driver::CompileOptions;

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
        let prover_file = package.root_dir.join(&args.prover_name).with_extension("toml");

        let cmd = noir_artifact_cli::commands::execute_cmd::ExecuteCommand {
            artifact: program_artifact_path,
            prover_file,
            output_dir: Some(workspace.target_directory_path()),
            witness_name: Some(
                args.witness_name.clone().unwrap_or_else(|| package.name.to_string()),
            ),
            contract_fn: None,
            oracle_file: None,
            oracle_resolver: args.oracle_resolver.clone(),
            oracle_root_dir: Some(workspace.root_dir.clone()),
            oracle_package_name: Some(package.name.to_string()),
            pedantic_solving: args.compile_options.pedantic_solving,
        };

        noir_artifact_cli::commands::execute_cmd::run(cmd)?;
    }
    Ok(())
}
