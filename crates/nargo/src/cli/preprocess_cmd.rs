use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use std::path::Path;

use crate::cli::compile_cmd::compile_circuit;
use crate::errors::CliError;

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("preprocess").unwrap();
    let allow_warnings = args.is_present("allow-warnings");
    preprocess(allow_warnings)
}

pub fn preprocess(allow_warnings: bool) -> Result<(), CliError> {
    let curr_dir = std::env::current_dir().unwrap();
    preprocess_with_path(curr_dir, allow_warnings)
}

fn preprocess_with_path<P: AsRef<Path>>(
    program_dir: P,
    allow_warnings: bool
) -> Result<(), CliError> {
    let compiled_program = compile_circuit(program_dir, false, allow_warnings)?;

    let backend = crate::backends::ConcreteBackend;

    backend.preprocess(compiled_program.circuit);
    Ok(())
}