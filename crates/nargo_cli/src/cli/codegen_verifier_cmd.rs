use super::fs::{create_named_dir, write_to_file};
use super::NargoConfig;
use crate::{cli::compile_cmd::compile_circuit, constants::CONTRACT_DIR, errors::CliError};
use acvm::SmartContract;
use clap::Args;
use noirc_driver::CompileOptions;

/// Generates a Solidity verifier smart contract for the program
#[derive(Debug, Clone, Args)]
pub(crate) struct CodegenVerifierCommand {
    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(args: CodegenVerifierCommand, config: NargoConfig) -> Result<(), CliError> {
    let compiled_program = compile_circuit(&config.program_dir, &args.compile_options)?;

    let backend = nargo::backends::ConcreteBackend;
    #[allow(deprecated)]
    let smart_contract_string = backend.eth_contract_from_cs(compiled_program.circuit);

    let contract_dir = config.program_dir.join(CONTRACT_DIR);
    create_named_dir(&contract_dir, "contract");
    let contract_path = contract_dir.join("plonk_vk").with_extension("sol");

    let path = write_to_file(smart_contract_string.as_bytes(), &contract_path);
    println!("Contract successfully created and located at {path}");
    Ok(())
}
