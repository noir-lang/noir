use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use std::path::{Path, PathBuf};

use crate::cli::compile_cmd::compile_circuit;
use crate::errors::CliError;

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("gates").unwrap();
    let show_ssa = args.is_present("show-ssa");
    let allow_warnings = args.is_present("allow-warnings");
    let program_dir =
        args.value_of("path").map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);

    count_gates_with_path(program_dir, show_ssa, allow_warnings)
}

pub fn count_gates_with_path<P: AsRef<Path>>(
    program_dir: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<(), CliError> {
    let compiled_program = compile_circuit(program_dir.as_ref(), show_ssa, allow_warnings)?;
    let num_opcodes = compiled_program.circuit.opcodes.len();
    let backend = crate::backends::ConcreteBackend;

    println!(
        "Total ACIR opcodes generated for language {:?}: {}",
        backend.np_language(),
        num_opcodes
    );

    let exact_circuit_size = backend.get_exact_circuit_size(compiled_program.circuit);
    println!("Backend circuit size: {exact_circuit_size}");

    Ok(())
}
