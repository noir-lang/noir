use acvm::ProofSystemCompiler;
use clap::Args;
use std::path::Path;

use crate::cli::compile_cmd::compile_circuit;
use crate::errors::CliError;

use super::NargoConfig;

/// Counts the occurrences of different gates in circuit
#[derive(Debug, Clone, Args)]
pub(crate) struct GatesCommand {
    /// Issue a warning for each unused variable instead of an error
    #[arg(short, long)]
    allow_warnings: bool,

    /// Emit debug information for the intermediate SSA IR
    #[arg(short, long)]
    show_ssa: bool,
}

pub(crate) fn run(args: GatesCommand, config: NargoConfig) -> Result<(), CliError> {
    count_gates_with_path(config.program_dir, args.show_ssa, args.allow_warnings)
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

    let exact_circuit_size = backend.get_exact_circuit_size(&compiled_program.circuit);
    println!("Backend circuit size: {exact_circuit_size}");

    Ok(())
}
