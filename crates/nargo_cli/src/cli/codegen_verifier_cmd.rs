use super::fs::{create_named_dir, program::read_program_from_file, write_to_file};
use super::NargoConfig;
use crate::{
    cli::compile_cmd::compile_circuit, constants::CONTRACT_DIR, constants::TARGET_DIR,
    errors::CliError,
};
use acvm::Backend;
use clap::Args;
use nargo::ops::{codegen_verifier, preprocess_program};
use noirc_driver::CompileOptions;

/// Generates a Solidity verifier smart contract for the program
#[derive(Debug, Clone, Args)]
pub(crate) struct CodegenVerifierCommand {
    /// The name of the circuit build files (ACIR, proving and verification keys)
    circuit_name: Option<String>,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: CodegenVerifierCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    // TODO(#1201): Should this be a utility function?
    let circuit_build_path = args
        .circuit_name
        .map(|circuit_name| config.program_dir.join(TARGET_DIR).join(circuit_name));

    let preprocessed_program = match circuit_build_path {
        Some(circuit_build_path) => read_program_from_file(circuit_build_path)?,
        None => {
            let compiled_program =
                compile_circuit(backend, config.program_dir.as_ref(), &args.compile_options)?;
            preprocess_program(backend, compiled_program)
                .map_err(CliError::ProofSystemCompilerError)?
        }
    };

    let smart_contract_string = codegen_verifier(backend, &preprocessed_program.verification_key)
        .map_err(CliError::SmartContractError)?;

    let contract_dir = config.program_dir.join(CONTRACT_DIR);
    create_named_dir(&contract_dir, "contract");
    let contract_path = contract_dir.join("plonk_vk").with_extension("sol");

    let path = write_to_file(smart_contract_string.as_bytes(), &contract_path);
    println!("Contract successfully created and located at {path}");
    Ok(())
}
