use acvm::ProofSystemCompiler;
use noirc_driver::{CompileOptions, CompiledProgram, Driver};
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

    /// Compile each contract function used within the program
    #[arg(short, long)]
    contracts: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(args: CompileCommand, config: NargoConfig) -> Result<(), CliError> {
    let mut circuit_dir = config.program_dir.clone();
    circuit_dir.push(TARGET_DIR);

    // If contracts is set we're compiling every function in a 'contract' rather than just 'main'.
    if args.contracts {
        let mut driver = setup_driver(&config.program_dir)?;
        let compiled_contracts = driver
            .compile_contracts(&args.compile_options)
            .map_err(|_| CliError::CompilationError)?;

        // Flatten each contract into a list of its functions, each being assigned a unique name.
        let compiled_programs = compiled_contracts.into_iter().flat_map(|contract| {
            let contract_id = format!("{}-{}", args.circuit_name, &contract.name);
            contract.functions.into_iter().map(move |(function, program)| {
                let program_name = format!("{}-{}", contract_id, function);
                (program_name, program)
            })
        });

        for (circuit_name, compiled_program) in compiled_programs {
            save_and_preprocess_program(&compiled_program, &circuit_name, &circuit_dir)?
        }
        Ok(())
    } else {
        let program = compile_circuit(&config.program_dir, &args.compile_options)?;
        save_and_preprocess_program(&program, &args.circuit_name, &circuit_dir)
    }
}

fn setup_driver(program_dir: &Path) -> Result<Driver, CliError> {
    let backend = crate::backends::ConcreteBackend;
    let mut driver = Resolver::resolve_root_config(program_dir, backend.np_language())?;
    add_std_lib(&mut driver);
    Ok(driver)
}

/// Save a program to disk along with proving and verification keys.
fn save_and_preprocess_program(
    compiled_program: &CompiledProgram,
    circuit_name: &str,
    circuit_dir: &Path,
) -> Result<(), CliError> {
    save_program_to_file(compiled_program, circuit_name, circuit_dir);
    preprocess_with_path(circuit_name, circuit_dir, &compiled_program.circuit)?;
    Ok(())
}

pub(crate) fn compile_circuit(
    program_dir: &Path,
    compile_options: &CompileOptions,
) -> Result<CompiledProgram, CliError> {
    let mut driver = setup_driver(program_dir)?;
    driver.compile_main(compile_options).map_err(|_| CliError::CompilationError)
}
