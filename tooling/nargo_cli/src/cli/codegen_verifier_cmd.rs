use super::NargoConfig;
use super::{
    compile_cmd::compile_bin_package,
    fs::{create_named_dir, write_to_file},
};
use crate::backends::Backend;
use crate::errors::CliError;

use acvm::acir::circuit::Opcode;
use acvm::Language;
use bb_abstraction_leaks::ACVM_BACKEND_BARRETENBERG;
use clap::Args;
use nargo::package::Package;
use nargo::workspace::Workspace;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::CompileOptions;
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
    let workspace = resolve_workspace_from_toml(&toml_path, selection)?;

    let (np_language, opcode_support) = backend.get_backend_info()?;
    for package in &workspace {
        let smart_contract_string = smart_contract_for_package(
            &workspace,
            backend,
            package,
            &args.compile_options,
            np_language,
            &|opcode| opcode_support.is_opcode_supported(opcode),
        )?;

        let contract_dir = workspace.contracts_directory_path(package);
        create_named_dir(&contract_dir, "contract");
        let contract_path = contract_dir.join("plonk_vk").with_extension("sol");

        let path = write_to_file(smart_contract_string.as_bytes(), &contract_path);
        println!("[{}] Contract successfully created and located at {path}", package.name);
    }

    Ok(())
}

fn smart_contract_for_package(
    workspace: &Workspace,
    backend: &Backend,
    package: &Package,
    compile_options: &CompileOptions,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> Result<String, CliError> {
    let program = compile_bin_package(
        workspace,
        package,
        compile_options,
        np_language,
        &is_opcode_supported,
    )?;

    let mut smart_contract_string = backend.eth_contract(&program.circuit)?;

    if backend.name() == ACVM_BACKEND_BARRETENBERG {
        smart_contract_string =
            bb_abstraction_leaks::complete_barretenberg_verifier_contract(smart_contract_string);
    }

    Ok(smart_contract_string)
}
