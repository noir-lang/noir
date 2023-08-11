use std::path::PathBuf;

use super::NargoConfig;
use super::{
    compile_cmd::compile_package,
    fs::{
        common_reference_string::{
            read_cached_common_reference_string, update_common_reference_string,
            write_cached_common_reference_string,
        },
        create_named_dir,
        program::read_program_from_file,
        write_to_file,
    },
};
use crate::errors::CliError;
use acvm::Backend;
use clap::Args;
use nargo::artifacts::program::PreprocessedProgram;
use nargo::{ops::codegen_verifier, package::Package};
use nargo_toml::{find_package_manifest, resolve_workspace_from_toml};
use noirc_driver::CompileOptions;
use noirc_frontend::graph::CrateName;

// TODO(#1388): pull this from backend.
const BACKEND_IDENTIFIER: &str = "acvm-backend-barretenberg";

/// Generates a Solidity verifier smart contract for the program
#[derive(Debug, Clone, Args)]
pub(crate) struct CodegenVerifierCommand {
    /// The name of the package to codegen
    #[clap(long)]
    package: Option<CrateName>,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: CodegenVerifierCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let toml_path = find_package_manifest(&config.program_dir)?;
    let workspace = resolve_workspace_from_toml(&toml_path, args.package)?;

    for package in &workspace {
        let circuit_build_path = workspace.package_build_path(package);

        let smart_contract_string = smart_contract_for_package(
            backend,
            package,
            circuit_build_path,
            &args.compile_options,
        )?;

        let contract_dir = workspace.contracts_directory_path(package);
        create_named_dir(&contract_dir, "contract");
        let contract_path = contract_dir.join("plonk_vk").with_extension("sol");

        let path = write_to_file(smart_contract_string.as_bytes(), &contract_path);
        println!("[{}] Contract successfully created and located at {path}", package.name);
    }

    Ok(())
}

fn smart_contract_for_package<B: Backend>(
    backend: &B,
    package: &Package,
    circuit_build_path: PathBuf,
    compile_options: &CompileOptions,
) -> Result<String, CliError<B>> {
    let preprocessed_program = if circuit_build_path.exists() {
        read_program_from_file(circuit_build_path)?
    } else {
        let (_, program) = compile_package(backend, package, compile_options)?;

        PreprocessedProgram {
            backend: String::from(BACKEND_IDENTIFIER),
            abi: program.abi,
            bytecode: program.circuit,
        }
    };

    let common_reference_string = read_cached_common_reference_string();
    let common_reference_string = update_common_reference_string(
        backend,
        &common_reference_string,
        &preprocessed_program.bytecode,
    )
    .map_err(CliError::CommonReferenceStringError)?;

    let (_, verification_key) = backend
        .preprocess(&common_reference_string, &preprocessed_program.bytecode)
        .map_err(CliError::ProofSystemCompilerError)?;

    let smart_contract_string = codegen_verifier(
        backend,
        &common_reference_string,
        &preprocessed_program.bytecode,
        &verification_key,
    )
    .map_err(CliError::SmartContractError)?;

    write_cached_common_reference_string(&common_reference_string);

    Ok(smart_contract_string)
}
