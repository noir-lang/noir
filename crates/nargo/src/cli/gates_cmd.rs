use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use std::path::Path;

use crate::cli::compile_cmd::compile_circuit;
use crate::errors::CliError;

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("gates").unwrap();
    let show_ssa = args.is_present("show-ssa");
    let allow_warnings = args.is_present("allow-warnings");
    count_gates(show_ssa, allow_warnings)
}

pub fn count_gates(show_ssa: bool, allow_warnings: bool) -> Result<(), CliError> {
    let curr_dir = std::env::current_dir().unwrap();
    count_gates_with_path(curr_dir, show_ssa, allow_warnings)
}

pub fn count_gates_with_path<P: AsRef<Path>>(
    program_dir: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<(), CliError> {
    let compiled_program = compile_circuit(program_dir.as_ref(), show_ssa, allow_warnings)?;
    let gates = compiled_program.circuit.gates.clone();
    let backend = crate::backends::ConcreteBackend;

    println!("Total gates generated for language {:?}: {}\n", backend.np_language(), gates.len());

    let exact_circuit_size = backend.get_exact_circuit_size(compiled_program.circuit);
    println!("\nBackend circuit size: {}\n", exact_circuit_size);

    Ok(())
}
