use super::fs::{create_named_dir, write_to_file};
use super::NargoConfig;
use crate::{cli::compile_cmd::compile_circuit, constants::CONTRACT_DIR, errors::CliError};
use acvm::SmartContract;
use clap::Args;
use noirc_driver::CompileOptions;

/// Generates a Solidity verifier smart contract for the program
#[derive(Debug, Clone, Args)]
pub(crate) struct ContractCommand {}

pub(crate) fn run(_args: ContractCommand, config: NargoConfig) -> Result<(), CliError> {
    let compiled_program =
        compile_circuit(config.program_dir.clone(), &CompileOptions::from(config.compile_options))?;

    let backend = crate::backends::ConcreteBackend;
    #[allow(deprecated)]
    let smart_contract_string = backend.eth_contract_from_cs(compiled_program.circuit);

    let mut contract_dir = config.program_dir;
    contract_dir.push(CONTRACT_DIR);
    let mut contract_path = create_named_dir(contract_dir.as_ref(), "contract");
    contract_path.push("plonk_vk");
    contract_path.set_extension("sol");

    let path = write_to_file(smart_contract_string.as_bytes(), &contract_path);
    println!("Contract successfully created and located at {path}");
    Ok(())
}
