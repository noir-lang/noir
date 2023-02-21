use acvm::acir::circuit::Circuit;
use acvm::ProofSystemCompiler;
use noirc_driver::CompileOptionsBuilder;
use std::path::{Path, PathBuf};

use clap::Args;

use crate::{constants::TARGET_DIR, errors::CliError, resolver::Resolver};

use super::fs::{acir::save_acir_to_dir, keys::save_key_to_dir};
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

    let circuit_path = compile_and_preprocess_circuit(
        &args.circuit_name,
        config.program_dir,
        circuit_path,
        args.allow_warnings,
    )?;

    println!("Generated ACIR code into {}", circuit_path.display());

    Ok(())
}

fn compile_and_preprocess_circuit<P: AsRef<Path>>(
    circuit_name: &str,
    program_dir: P,
    circuit_dir: P,
    allow_warnings: bool,
) -> Result<PathBuf, CliError> {
    let compiled_program = compile_circuit(program_dir, false, allow_warnings)?;
    let circuit_path = save_acir_to_dir(&compiled_program.circuit, circuit_name, &circuit_dir);

    preprocess_with_path(circuit_name, circuit_dir, compiled_program.circuit)?;

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

    let config = CompileOptionsBuilder::default()
        .allow_warnings(allow_warnings)
        .show_ssa(show_ssa)
        .build()
        .unwrap();
    driver.into_compiled_program(&config).map_err(|_| std::process::exit(1))
}

fn preprocess_with_path<P: AsRef<Path>>(
    key_name: &str,
    preprocess_dir: P,
    circuit: Circuit,
) -> Result<(PathBuf, PathBuf), CliError> {
    let backend = crate::backends::ConcreteBackend;

    let (proving_key, verification_key) = backend.preprocess(circuit);

    let pk_path = save_key_to_dir(proving_key, key_name, &preprocess_dir, true)?;
    println!("Proving key saved to {}", pk_path.display());
    let vk_path = save_key_to_dir(verification_key, key_name, preprocess_dir, false)?;
    println!("Verification key saved to {}", vk_path.display());

    Ok((pk_path, vk_path))
}
