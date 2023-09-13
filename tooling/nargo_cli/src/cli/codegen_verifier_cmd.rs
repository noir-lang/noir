use std::path::PathBuf;

use super::NargoConfig;
use super::{
    compile_cmd::compile_bin_package,
    fs::{create_named_dir, program::read_program_from_file, write_to_file},
};
use crate::backends::Backend;
use crate::errors::CliError;

use acvm::acir::circuit::Opcode;
use acvm::Language;
use clap::Args;
use nargo::artifacts::program::PreprocessedProgram;
use nargo::package::Package;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::CompileOptions;
use noirc_frontend::graph::CrateName;

// TODO(#1388): pull this from backend.
const BACKEND_IDENTIFIER: &str = "acvm-backend-barretenberg";

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

    let (np_language, is_opcode_supported) = backend.get_backend_info()?;
    for package in &workspace {
        if !package.is_binary() {
            // Contract and library packages cannot generate verifiers.
            continue;
        }

        let circuit_build_path = workspace.package_build_path(package);

        let smart_contract_string = smart_contract_for_package(
            backend,
            package,
            circuit_build_path,
            &args.compile_options,
            np_language,
            &is_opcode_supported,
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
    backend: &Backend,
    package: &Package,
    circuit_build_path: PathBuf,
    compile_options: &CompileOptions,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> Result<String, CliError> {
    let preprocessed_program = if circuit_build_path.exists() {
        read_program_from_file(circuit_build_path)?
    } else {
        let program =
            compile_bin_package(package, compile_options, np_language, &is_opcode_supported)?;

        PreprocessedProgram {
            backend: String::from(BACKEND_IDENTIFIER),
            abi: program.abi,
            bytecode: program.circuit,
        }
    };

    let smart_contract_string = backend.eth_contract(&preprocessed_program.bytecode)?;

    Ok(smart_contract_string)
}
