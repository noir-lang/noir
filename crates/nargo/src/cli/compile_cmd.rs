use acvm::acir::circuit::Circuit;
use acvm::ProofSystemCompiler;
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

    /// Issue a warning for each unused variable instead of an error
    #[arg(short, long)]
    allow_warnings: bool,

    /// Compile each contract function used within the program
    #[arg(short, long)]
    contracts: bool,
}

pub(crate) fn run(mut args: CompileCommand, config: NargoConfig) -> Result<(), CliError> {
    let driver = check_crate(&config.program_dir, args.allow_warnings)?;

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
        let main = driver.main_function();
        compile_and_save_program(&driver, main, &args, &circuit_dir)
    }
}

fn setup_driver(program_dir: impl AsRef<Path>) -> Result<Driver, CliError> {
    let backend = crate::backends::ConcreteBackend;
    let mut driver = Resolver::resolve_root_config(program_dir.as_ref(), backend.np_language())?;
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
        .compile_no_check(false, args.allow_warnings, main, true)
        .map_err(|_| CliError::Generic(format!("'{}' failed to compile", args.circuit_name)))?;

    let circuit_path = save_program_to_file(&compiled_program, &args.circuit_name, circuit_dir);

    preprocess_with_path(&args.circuit_name, circuit_dir, &compiled_program.circuit)?;

    println!("Generated ACIR code into {}", circuit_path.display());
    Ok(())
}

pub(crate) fn compile_circuit<P: AsRef<Path>>(
    program_dir: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<noirc_driver::CompiledProgram, CliError> {
    let mut driver = setup_driver(program_dir)?;
    driver.compile_main(show_ssa, allow_warnings, true).map_err(|_| CliError::CompilationError)
}

fn check_crate(program_dir: impl AsRef<Path>, allow_warnings: bool) -> Result<Driver, CliError> {
    let mut driver = setup_driver(program_dir)?;
    driver.check_crate(allow_warnings).map_err(|_| CliError::CompilationError)?;
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
    println!("Proving key saved to {}", pk_path.display());
    let vk_path = save_key_to_dir(verification_key, key_name, preprocess_dir, false)?;
    println!("Verification key saved to {}", vk_path.display());

    Ok((pk_path, vk_path))
}
