use super::fs::{
    common_reference_string::{
        read_cached_common_reference_string, update_common_reference_string,
        write_cached_common_reference_string,
    },
    create_named_dir,
    program::read_program_from_file,
    write_to_file,
};
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

    let common_reference_string = read_cached_common_reference_string();

    let (common_reference_string, preprocessed_program) = match circuit_build_path {
        Some(circuit_build_path) => {
            let program = read_program_from_file(circuit_build_path)?;
            let common_reference_string = update_common_reference_string(
                backend,
                &common_reference_string,
                &program.bytecode,
            )
            .map_err(CliError::CommonReferenceStringError)?;
            (common_reference_string, program)
        }
        None => {
            let (program, _) =
                compile_circuit(backend, None, config.program_dir.as_ref(), &args.compile_options)?;
            let common_reference_string =
                update_common_reference_string(backend, &common_reference_string, &program.circuit)
                    .map_err(CliError::CommonReferenceStringError)?;
            let (program, _) = preprocess_program(backend, true, &common_reference_string, program)
                .map_err(CliError::ProofSystemCompilerError)?;
            (common_reference_string, program)
        }
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

    let contract_dir = config.program_dir.join(CONTRACT_DIR);
    create_named_dir(&contract_dir, "contract");
    let contract_path = contract_dir.join("plonk_vk").with_extension("sol");

    let path = write_to_file(smart_contract_string.as_bytes(), &contract_path);
    println!("Contract successfully created and located at {path}");
    Ok(())
}
