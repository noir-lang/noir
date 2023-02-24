use acvm::ProofSystemCompiler;
use std::path::Path;

use clap::Args;

use crate::{constants::TARGET_DIR, errors::CliError, resolver::Resolver};

use super::fs::program::save_program_to_file;
use super::preprocess_cmd::preprocess_with_path;
use super::{add_std_lib, NargoConfig};

/// Compile the program and its secret execution trace into ACIR format
#[derive(Debug, Clone, Args)]
pub(crate) struct CompileCommand {
    /// The name of the ACIR file
    circuit_name: String,

    /// Issue a warning for each unused variable instead of an error
    #[arg(short, long)]
    allow_warnings: bool,
}

pub(crate) fn run(args: CompileCommand, config: NargoConfig) -> Result<(), CliError> {
    let mut circuit_path = config.program_dir.clone();
    circuit_path.push(TARGET_DIR);

    let compiled_program = compile_circuit(config.program_dir, false, args.allow_warnings)?;

    save_program_to_file(&compiled_program, &args.circuit_name, &circuit_path);

    preprocess_with_path(&args.circuit_name, circuit_path, &compiled_program.circuit)?;

    Ok(())
}

pub(crate) fn compile_circuit<P: AsRef<Path>>(
    program_dir: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<noirc_driver::CompiledProgram, CliError> {
    let backend = crate::backends::ConcreteBackend;
    let mut driver = Resolver::resolve_root_config(program_dir.as_ref(), backend.np_language())?;
    add_std_lib(&mut driver);

    driver.into_compiled_program(show_ssa, allow_warnings).map_err(|_| std::process::exit(1))
}
