use super::{create_dir, write_to_file, CONTRACT_DIR};
use crate::{errors::CliError, resolver::Resolver};
use acvm::SmartContract;
use clap::ArgMatches;
use std::path::PathBuf;

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let cmd = args.subcommand_matches("contract").unwrap();

    let package_dir = match cmd.value_of("path") {
        Some(path) => std::path::PathBuf::from(path),
        None => std::env::current_dir().unwrap(),
    };
    let driver = Resolver::resolve_root_config(&package_dir)?;
    let compiled_program = driver.into_compiled_program();

    let backend = acvm::ConcreteBackend;
    let smart_contract_string = backend.eth_contract_from_cs(compiled_program.circuit);

    let mut contract_path = create_contract_dir();
    contract_path.push("plonk_vk");
    contract_path.set_extension("sol");

    let path = write_to_file(&smart_contract_string.as_bytes(), &contract_path);
    println!("Contract successfully created and located at {}", path);
    Ok(())
}

fn create_contract_dir() -> PathBuf {
    create_dir(CONTRACT_DIR).expect("could not create the `contract` directory")
}
