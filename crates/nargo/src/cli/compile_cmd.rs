use acvm::acir::circuit::Circuit;
use acvm::ProofSystemCompiler;
use noirc_driver::CompileOptions;
use noirc_driver::Driver;
use noirc_frontend::node_interner::FuncId;
use std::path::{Path, PathBuf};

use clap::Args;

use crate::{constants::TARGET_DIR, errors::CliError, resolver::Resolver};

use super::fs::{keys::save_key_to_dir, program::save_program_to_file};
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

pub(crate) fn run(mut args: CompileCommand, config: NargoConfig) -> Result<(), CliError> {
    let driver = check_crate(&config.program_dir, &args.compile_options)?;

    let mut circuit_dir = config.program_dir;
    circuit_dir.push(TARGET_DIR);

    // If contracts is set we're compiling every function in a 'contract' rather than just 'main'.
    if args.contracts {
        let circuit_name = args.circuit_name.clone();

        for contract in driver.get_all_contracts() {
            for function in contract.functions {
                let name = driver.function_name(function);
                args.circuit_name = format!("{}-{}-{name}", circuit_name, &contract.name);
                compile_and_save_program(&driver, function, &args, &circuit_dir)?;
            }
        }
        Ok(())
    } else {
        let main = driver.main_function().map_err(|_| CliError::CompilationError)?;
        compile_and_save_program(&driver, main, &args, &circuit_dir)
    }
}

fn setup_driver(program_dir: &Path) -> Result<Driver, CliError> {
    let backend = crate::backends::ConcreteBackend;
    let mut driver = Resolver::resolve_root_config(program_dir, backend.np_language())?;
    add_std_lib(&mut driver);
    Ok(driver)
}

/// Compile and save a program to disk with the given main function.
fn compile_and_save_program(
    driver: &Driver,
    main: FuncId,
    args: &CompileCommand,
    circuit_dir: &Path,
) -> Result<(), CliError> {
    let compiled_program = driver
        .compile_no_check(&args.compile_options, main)
        .map_err(|_| CliError::Generic(format!("'{}' failed to compile", args.circuit_name)))?;

    save_program_to_file(&compiled_program, &args.circuit_name, circuit_dir);

    preprocess_with_path(&args.circuit_name, circuit_dir, &compiled_program.circuit)?;
    Ok(())
}

pub(crate) fn compile_circuit(
    program_dir: &Path,
    compile_options: &CompileOptions,
) -> Result<noirc_driver::CompiledProgram, CliError> {
    let mut driver = setup_driver(program_dir)?;
    driver.compile_main(compile_options).map_err(|_| CliError::CompilationError)
}

fn check_crate(program_dir: &Path, options: &CompileOptions) -> Result<Driver, CliError> {
    let mut driver = setup_driver(program_dir)?;
    driver.check_crate(options).map_err(|_| CliError::CompilationError)?;
    Ok(driver)
}

fn preprocess_with_path<P: AsRef<Path>>(
    key_name: &str,
    preprocess_dir: P,
    circuit: &Circuit,
) -> Result<(PathBuf, PathBuf), CliError> {
    let backend = crate::backends::ConcreteBackend;

    let (proving_key, verification_key) = backend.preprocess(circuit);

    let pk_path = save_key_to_dir(proving_key, key_name, &preprocess_dir, true)?;
    let vk_path = save_key_to_dir(verification_key, key_name, preprocess_dir, false)?;

    Ok((pk_path, vk_path))
}
