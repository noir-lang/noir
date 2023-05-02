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

pub(crate) fn run(args: PrintAcirCommand, config: NargoConfig) -> Result<(), CliError> {
    print_acir_with_path(config.program_dir, &args.compile_options)
}

fn print_acir_with_path<P: AsRef<Path>>(
    program_dir: P,
    compile_options: &CompileOptions,
) -> Result<(), CliError> {
    let backend = crate::backends::ConcreteBackend::default();

    let compiled_program = compile_circuit(&backend, program_dir.as_ref(), compile_options)?;
    println!("{}", compiled_program.circuit);

    Ok(())
}
