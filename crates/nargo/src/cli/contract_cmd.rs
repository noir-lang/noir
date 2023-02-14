use std::path::PathBuf;

use super::{create_named_dir, write_to_file};
use crate::{cli::compile_cmd::compile_circuit, constants::CONTRACT_DIR, errors::CliError};
use acvm::SmartContract;
use clap::ArgMatches;

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("contract").unwrap();

    let allow_warnings = args.is_present("allow-warnings");
    let program_dir =
        args.value_of("path").map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);

    let compiled_program = compile_circuit(program_dir.clone(), false, allow_warnings)?;

    let backend = crate::backends::ConcreteBackend;
    let smart_contract_string = backend.eth_contract_from_cs(compiled_program.circuit);

    let mut contract_dir = program_dir;
    contract_dir.push(CONTRACT_DIR);
    let mut contract_path = create_named_dir(contract_dir.as_ref(), "contract");
    contract_path.push("plonk_vk");
    contract_path.set_extension("sol");

    let path = write_to_file(smart_contract_string.as_bytes(), &contract_path);
    println!("Contract successfully created and located at {path}");
    Ok(())
}
