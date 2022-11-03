use std::path::PathBuf;

use acvm::acir::native_types::Witness;
use acvm::ProofSystemCompiler;

use clap::ArgMatches;

use std::path::Path;

use crate::{errors::CliError, resolver::Resolver};

use super::{create_named_dir, write_to_file, BUILD_DIR};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("compile").unwrap();
    let circuit_name = args.value_of("circuit_name").unwrap();
    let witness = args.is_present("witness");

    let curr_dir = std::env::current_dir().unwrap();
    let mut circuit_path = PathBuf::new();
    circuit_path.push(BUILD_DIR);

    let result =
        generate_circuit_and_witness_to_disk(circuit_name, curr_dir, circuit_path, witness);
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn generate_circuit_and_witness_to_disk<P: AsRef<Path>>(
    circuit_name: &str,
    program_dir: P,
    circuit_dir: P,
    generate_witness: bool,
) -> Result<PathBuf, CliError> {
    let compiled_program = compile_circuit(program_dir.as_ref(), false)?;
    let serialized = compiled_program.circuit.to_bytes();

    let mut circuit_path = create_named_dir(circuit_dir.as_ref(), "build");
    circuit_path.push(circuit_name);
    circuit_path.set_extension(crate::cli::ACIR_EXT);
    let path = write_to_file(serialized.as_slice(), &circuit_path);
    println!("Generated ACIR code into {}", path);
    println!("{:?}", std::fs::canonicalize(&circuit_path));

    if generate_witness {
        let solved_witness = super::prove_cmd::solve_witness(program_dir, &compiled_program)?;
        let buf = Witness::to_bytes(&solved_witness);

        circuit_path.pop();
        circuit_path.push(circuit_name);
        circuit_path.set_extension(crate::cli::WITNESS_EXT);
        write_to_file(buf.as_slice(), &circuit_path);
    }

    Ok(circuit_path)
}

pub fn compile_circuit<P: AsRef<Path>>(
    program_dir: P,
    show_ssa: bool,
) -> Result<noirc_driver::CompiledProgram, CliError> {
    let backend = crate::backends::ConcreteBackend;
    let driver = Resolver::resolve_root_config(program_dir.as_ref(), backend.np_language())?;
    
    let compiled_program = driver.into_compiled_program(backend.np_language(), show_ssa);

    Ok(compiled_program)
}
