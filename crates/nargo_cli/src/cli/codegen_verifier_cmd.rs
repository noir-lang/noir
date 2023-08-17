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
use nargo::{
    ops::{codegen_verifier, preprocess_program},
    package::Package,
};
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

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: CodegenVerifierCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(&toml_path, selection)?;

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
    let common_reference_string = read_cached_common_reference_string();
    let (common_reference_string, preprocessed_program) = if circuit_build_path.exists() {
        let program = read_program_from_file(circuit_build_path)?;
        let common_reference_string =
            update_common_reference_string(backend, &common_reference_string, &program.bytecode)
                .map_err(CliError::CommonReferenceStringError)?;
        (common_reference_string, program)
    } else {
        let (_, program) = compile_package(backend, package, compile_options)?;
        let common_reference_string =
            update_common_reference_string(backend, &common_reference_string, &program.circuit)
                .map_err(CliError::CommonReferenceStringError)?;
        let (program, _) = preprocess_program(backend, true, &common_reference_string, program)
            .map_err(CliError::ProofSystemCompilerError)?;
        (common_reference_string, program)
    };

    let verification_key = preprocessed_program
        .verification_key
        .expect("Verification key should exist as `true` is passed to `preprocess_program`");
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
