use acvm::ProofSystemCompiler;
use iter_extended::vecmap;
use noirc_driver::{CompileOptions, CompiledProgram, Driver};
use std::path::Path;

use clap::Args;

use crate::preprocess::{preprocess_contract, preprocess_program};
use crate::resolver::DependencyResolutionError;
use crate::{constants::TARGET_DIR, errors::CliError, resolver::Resolver};

use super::fs::program::{save_contract_to_file, save_program_to_file};
use super::NargoConfig;

/// Compile the program and its secret execution trace into ACIR format
#[derive(Debug, Clone, Args)]
pub(crate) struct CompileCommand {
    /// The name of the ACIR file
    circuit_name: String,

    /// Compile each contract function used within the program
    #[arg(short, long)]
    contracts: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(args: CompileCommand, config: NargoConfig) -> Result<(), CliError> {
    let circuit_dir = config.program_dir.join(TARGET_DIR);

    // If contracts is set we're compiling every function in a 'contract' rather than just 'main'.
    if args.contracts {
        let mut driver = setup_driver(&config.program_dir)?;
        let compiled_contracts = driver
            .compile_contracts(&args.compile_options)
            .map_err(|_| CliError::CompilationError)?;
        let preprocessed_contracts = vecmap(compiled_contracts, preprocess_contract);
        for contract in preprocessed_contracts {
            save_contract_to_file(&contract, &args.circuit_name, &circuit_dir);
        }
    } else {
        let program = compile_circuit(&config.program_dir, &args.compile_options)?;
        let preprocessed_program = preprocess_program(program);
        save_program_to_file(&preprocessed_program, &args.circuit_name, circuit_dir);
    }
    Ok(())
}

fn setup_driver(program_dir: &Path) -> Result<Driver, DependencyResolutionError> {
    let backend = crate::backends::ConcreteBackend;
    Resolver::resolve_root_manifest(program_dir, backend.np_language())
}

pub(crate) fn compile_circuit(
    program_dir: &Path,
    compile_options: &CompileOptions,
) -> Result<CompiledProgram, CliError> {
    let mut driver = setup_driver(program_dir)?;
    driver.compile_main(compile_options).map_err(|_| CliError::CompilationError)
}
