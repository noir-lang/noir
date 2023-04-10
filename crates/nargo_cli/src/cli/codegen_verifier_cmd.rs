use super::fs::{
    create_named_dir, keys::fetch_pk_and_vk, program::read_program_from_file, write_to_file,
};
use super::NargoConfig;
use crate::{
    cli::compile_cmd::compile_circuit, constants::CONTRACT_DIR, constants::TARGET_DIR,
    errors::CliError,
};
use acvm::{ProofSystemCompiler, SmartContract};
use clap::Args;
use noirc_driver::CompileOptions;

/// Generates a Solidity verifier smart contract for the program
#[derive(Debug, Clone, Args)]
pub(crate) struct CodegenVerifierCommand {
    /// The name of the circuit build files (ACIR, proving and verification keys)
    circuit_name: Option<String>,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(args: CodegenVerifierCommand, config: NargoConfig) -> Result<(), CliError> {
    let backend = crate::backends::ConcreteBackend;

    // Based on code in verify_cmd.rs
    // TODO(blaine): Should this be a utility function?
    let circuit_build_path = args
        .circuit_name
        .map(|circuit_name| config.program_dir.join(TARGET_DIR).join(circuit_name));

    let verification_key = match circuit_build_path {
        Some(circuit_build_path) => {
            let compiled_program = read_program_from_file(&circuit_build_path)?;

            let (_, verification_key) =
                fetch_pk_and_vk(&compiled_program.circuit, circuit_build_path, false, true)?;
            verification_key
        }
        None => {
            let compiled_program =
                compile_circuit(config.program_dir.as_ref(), &args.compile_options)?;

            let (_, verification_key) = backend.preprocess(&compiled_program.circuit);
            verification_key
        }
    };

    let smart_contract_string = backend.eth_contract_from_vk(&verification_key);

    let contract_dir = config.program_dir.join(CONTRACT_DIR);
    create_named_dir(&contract_dir, "contract");
    let contract_path = contract_dir.join("plonk_vk").with_extension("sol");

    let path = write_to_file(smart_contract_string.as_bytes(), &contract_path);
    println!("Contract successfully created and located at {path}");
    Ok(())
}
