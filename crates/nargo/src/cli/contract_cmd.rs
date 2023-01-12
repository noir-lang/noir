use super::{create_named_dir, write_to_file, CONTRACT_DIR};
use crate::{cli::compile_cmd::compile_circuit, errors::CliError};
use acvm::SmartContract;
use clap::ArgMatches;

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let cmd = args.subcommand_matches("contract").unwrap();

    let package_dir = match cmd.value_of("path") {
        Some(path) => std::path::PathBuf::from(path),
        None => std::env::current_dir().unwrap(),
    };

    let allow_warnings = args.is_present("allow-warnings");
    let compiled_program = compile_circuit(package_dir, false, allow_warnings)?;

    let backend = crate::backends::ConcreteBackend;
    let smart_contract_string = backend.eth_contract_from_cs(compiled_program.circuit);

    let mut contract_path = create_named_dir(CONTRACT_DIR.as_ref(), "contract");
    contract_path.push("plonk_vk");
    contract_path.set_extension("sol");

    let path = write_to_file(smart_contract_string.as_bytes(), &contract_path);
    println!("Contract successfully created and located at {}", path);
    Ok(())
}
