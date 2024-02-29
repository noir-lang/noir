use super::fs::{create_named_dir, write_to_file};
use super::NargoConfig;
use crate::backends::Backend;
use crate::cli::compile_cmd::report_errors;
use crate::errors::CliError;

use clap::Args;
use nargo::ops::compile_program;
use nargo::{insert_all_files_for_workspace_into_file_manager, parse_all};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{file_manager_with_stdlib, CompileOptions, NOIR_ARTIFACT_VERSION_STRING};
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

    let mut workspace_file_manager = file_manager_with_stdlib(&workspace.root_dir);
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = parse_all(&workspace_file_manager);

    let expression_width = backend.get_backend_info()?;
    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());
    for package in binary_packages {
        let compilation_result = compile_program(
            &workspace_file_manager,
            &parsed_files,
            package,
            &args.compile_options,
            None,
        );

        let program = report_errors(
            compilation_result,
            &workspace_file_manager,
            args.compile_options.deny_warnings,
            args.compile_options.silence_warnings,
        )?;

        let program = nargo::ops::transform_program(program, expression_width);

        let smart_contract_string = backend.eth_contract(&program.circuit)?;

        let contract_dir = workspace.contracts_directory_path(package);
        create_named_dir(&contract_dir, "contract");
        let contract_path = contract_dir.join("plonk_vk").with_extension("sol");

        let path = write_to_file(smart_contract_string.as_bytes(), &contract_path);
        println!("[{}] Contract successfully created and located at {path}", package.name);
    }

    Ok(())
}
