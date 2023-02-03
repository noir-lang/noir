use std::path::PathBuf;

use acvm::acir::native_types::Witness;
use acvm::ProofSystemCompiler;

use clap::ArgMatches;

use std::path::Path;

use crate::{
    constants::{ACIR_EXT, TARGET_DIR, WITNESS_EXT},
    errors::CliError,
    resolver::Resolver,
};

use super::{add_std_lib, create_named_dir, write_to_file};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("compile").unwrap();
    let circuit_name = args.value_of("circuit_name").unwrap();
    let witness = args.is_present("witness");
    let allow_warnings = args.is_present("allow-warnings");

    let current_dir = std::env::current_dir().unwrap();
    let mut circuit_path = PathBuf::new();
    circuit_path.push(TARGET_DIR);

    generate_circuit_and_witness_to_disk(
        circuit_name,
        current_dir,
        circuit_path,
        witness,
        allow_warnings,
    )
    .map(|_| ())
}

#[allow(deprecated)]
pub fn generate_circuit_and_witness_to_disk<P: AsRef<Path>>(
    circuit_name: &str,
    program_dir: P,
    circuit_dir: P,
    generate_witness: bool,
    allow_warnings: bool,
) -> Result<PathBuf, CliError> {
    let compiled_program = compile_circuit(program_dir.as_ref(), false, allow_warnings)?;
    let serialized = compiled_program.circuit.to_bytes();

    let mut circuit_path = create_named_dir(circuit_dir.as_ref(), "build");
    circuit_path.push(circuit_name);
    circuit_path.set_extension(ACIR_EXT);
    let path = write_to_file(serialized.as_slice(), &circuit_path);
    println!("Generated ACIR code into {path}");
    println!("{:?}", std::fs::canonicalize(&circuit_path));

    if generate_witness {
        let solved_witness =
            super::prove_cmd::parse_and_solve_witness(program_dir, &compiled_program)?;
        let buf = Witness::to_bytes(&solved_witness);

        circuit_path.pop();
        circuit_path.push(circuit_name);
        circuit_path.set_extension(WITNESS_EXT);
        write_to_file(buf.as_slice(), &circuit_path);
    }

    Ok(circuit_path)
}

pub fn compile_circuit<P: AsRef<Path>>(
    program_dir: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<noirc_driver::CompiledProgram, CliError> {
    let backend = crate::backends::ConcreteBackend;
    let mut driver = Resolver::resolve_root_config(program_dir.as_ref(), backend.np_language())?;
    add_std_lib(&mut driver);

    driver
        .into_compiled_program(backend.np_language(), show_ssa, allow_warnings)
        .map_err(|_| std::process::exit(1))
}
