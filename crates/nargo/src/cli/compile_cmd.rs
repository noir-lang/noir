use std::path::PathBuf;

use acvm::acir::native_types::Witness;

use clap::ArgMatches;

use std::path::Path;

use crate::errors::CliError;

use super::{create_named_dir, write_to_file, BUILD_DIR};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("compile").unwrap();
    let circuit_name = args.value_of("circuit_name").unwrap();

    let curr_dir = std::env::current_dir().unwrap();
    let mut circuit_path = PathBuf::new();
    circuit_path.push(BUILD_DIR);
    let result = generate_circuit_to_disk(circuit_name, curr_dir, circuit_path);
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn generate_circuit_to_disk<P: AsRef<Path>>(
    circuit_name: &str,
    program_dir: P,
    circuit_dir: P,
) -> Result<PathBuf, CliError> {
    let (compiled_program, solved_witness) =
        super::prove_cmd::compile_circuit_and_witness(program_dir, false)?;
    let serialized = compiled_program.circuit.to_bytes();
    let buf = Witness::to_bytes(&solved_witness);

    let mut circuit_path = create_named_dir(circuit_dir.as_ref(), "build");
    circuit_path.push(circuit_name);
    circuit_path.set_extension(crate::cli::ACIR_EXT);
    let path = write_to_file(serialized.as_slice(), &circuit_path);
    println!("Generated ACIR code into {}", path);
    println!("{:?}", std::fs::canonicalize(&circuit_path));

    circuit_path.pop();
    circuit_path.push(circuit_name);
    circuit_path.set_extension(crate::cli::WITNESS_EXT);
    write_to_file(buf.as_slice(), &circuit_path);

    Ok(circuit_path)
}
