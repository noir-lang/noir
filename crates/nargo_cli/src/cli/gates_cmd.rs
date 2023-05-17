use acvm::Backend;
use clap::Args;
use noirc_driver::CompileOptions;
use std::path::Path;

use crate::cli::compile_cmd::compile_circuit;
use crate::errors::CliError;

use super::NargoConfig;

/// Counts the occurrences of different gates in circuit
#[derive(Debug, Clone, Args)]
pub(crate) struct GatesCommand {
    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: GatesCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    count_gates_with_path(backend, config.program_dir, &args.compile_options)
}

fn count_gates_with_path<B: Backend, P: AsRef<Path>>(
    backend: &B,
    program_dir: P,
    compile_options: &CompileOptions,
) -> Result<(), CliError<B>> {
    let compiled_program = compile_circuit(backend, program_dir.as_ref(), compile_options)?;
    let num_opcodes = compiled_program.circuit.opcodes.len();

    println!(
        "Total ACIR opcodes generated for language {:?}: {}",
        backend.np_language(),
        num_opcodes
    );

    let exact_circuit_size = backend
        .get_exact_circuit_size(&compiled_program.circuit)
        .map_err(CliError::ProofSystemCompilerError)?;
    println!("Backend circuit size: {exact_circuit_size}");

    Ok(())
}
