use super::compile_cmd::compile_workspace_full;
use super::fs::{create_named_dir, write_to_file};
use super::NargoConfig;
use crate::backends::Backend;
use crate::cli::fs::program::read_program_from_file;
use crate::errors::CliError;

use clap::Args;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{CompileOptions, NOIR_ARTIFACT_VERSION_STRING};
use noirc_frontend::graph::CrateName;

/// Generates a Solidity verifier smart contract for the program
#[derive(Debug, Clone, Args)]
pub(crate) struct CodegenVerifierCommand {
    /// The name of the package to codegen
    #[clap(long, conflicts_with = "workspace")]
    package: Option<CrateName>,

    /// Codegen all packages in the workspace
    #[clap(long, conflicts_with = "package")]
    workspace: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(
    backend: &Backend,
    args: CodegenVerifierCommand,
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

    // Compile the full workspace in order to generate any build artifacts.
    compile_workspace_full(&workspace, &args.compile_options)?;

    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());
    for package in binary_packages {
        let program_artifact_path = workspace.package_build_path(package);
        let program = read_program_from_file(&program_artifact_path)?;

        // TODO(https://github.com/noir-lang/noir/issues/4428):
        // We do not expect to have a smart contract verifier for a foldable program with multiple circuits.
        // However, in the future we can expect to possibly have non-inlined ACIR functions during compilation
        // that will be inlined at a later step such as by the ACVM compiler or by the backend.
        // Add appropriate handling here once the compiler enables multiple ACIR functions.
        assert_eq!(program.bytecode.functions.len(), 1);
        let smart_contract_string = backend.eth_contract(program_artifact_path)?;

        let contract_dir = workspace.contracts_directory_path(package);
        create_named_dir(&contract_dir, "contract");
        let contract_path = contract_dir.join("plonk_vk").with_extension("sol");

        let path = write_to_file(smart_contract_string.as_bytes(), &contract_path);
        println!("[{}] Contract successfully created and located at {path}", package.name);
    }

    Ok(())
}
