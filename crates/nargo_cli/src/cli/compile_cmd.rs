use acvm::ProofSystemCompiler;
use iter_extended::try_vecmap;
use noirc_driver::{CompileOptions, CompiledProgram, Driver};
use std::path::Path;

use clap::Args;

use nargo::ops::{preprocess_contract, preprocess_program};

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

    let backend = crate::backends::ConcreteBackend::default();

    // If contracts is set we're compiling every function in a 'contract' rather than just 'main'.
    if args.contracts {
        let mut driver = setup_driver(&backend, &config.program_dir)?;
        let compiled_contracts = driver
            .compile_contracts(&args.compile_options)
            .map_err(|_| CliError::CompilationError)?;
        let preprocessed_contracts =
            try_vecmap(compiled_contracts, |contract| preprocess_contract(&backend, contract))?;
        for contract in preprocessed_contracts {
            save_contract_to_file(
                &contract,
                &format!("{}-{}", &args.circuit_name, contract.name),
                &circuit_dir,
            );
        }
    } else {
        let program = compile_circuit(&backend, &config.program_dir, &args.compile_options)?;
        let preprocessed_program = preprocess_program(&backend, program)?;
        save_program_to_file(&preprocessed_program, &args.circuit_name, circuit_dir);
    }
    Ok(())
}

fn setup_driver(
    backend: &impl ProofSystemCompiler,
    program_dir: &Path,
) -> Result<Driver, DependencyResolutionError> {
    Resolver::resolve_root_manifest(program_dir, backend.np_language())
}

pub(crate) fn compile_circuit(
    backend: &impl ProofSystemCompiler,
    program_dir: &Path,
    compile_options: &CompileOptions,
) -> Result<CompiledProgram, CliError> {
    let mut driver = setup_driver(backend, program_dir)?;
    driver.compile_main(compile_options).map_err(|_| CliError::CompilationError)
}
