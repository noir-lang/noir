use acvm::Backend;
use clap::Args;
use noirc_driver::CompileOptions;
use std::path::Path;

use crate::cli::compile_cmd::compile_circuit;
use crate::errors::CliError;

use super::NargoConfig;

/// Prints out the ACIR for a compiled circuit
#[derive(Debug, Clone, Args)]
pub(crate) struct PrintAcirCommand {
    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: PrintAcirCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    print_acir_with_path(backend, config.program_dir, &args.compile_options)
}

fn print_acir_with_path<B: Backend, P: AsRef<Path>>(
    backend: &B,
    program_dir: P,
    compile_options: &CompileOptions,
) -> Result<(), CliError<B>> {
    let compiled_program = compile_circuit(backend, program_dir.as_ref(), compile_options)?;
    println!("{}", compiled_program.circuit);

    Ok(())
}
