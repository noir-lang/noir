use acvm::{acir::circuit::Circuit, ProofSystemCompiler};
use iter_extended::{try_btree_map, try_vecmap};
use noirc_driver::{CompileOptions, CompiledProgram, Driver};
use noirc_frontend::{hir::def_map::Contract, node_interner::FuncId};
use std::collections::BTreeMap;
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

struct CompiledContract {
    /// The name of the contract.
    name: String,
    /// Each of the contract's functions are compiled into a separate `CompiledProgram`
    /// stored in this `BTreeMap`.
    functions: BTreeMap<String, CompiledProgram>,
}

pub(crate) fn run(args: CompileCommand, config: NargoConfig) -> Result<(), CliError> {
    let driver = check_crate(&config.program_dir, &args.compile_options)?;

    let mut circuit_dir = config.program_dir;
    circuit_dir.push(TARGET_DIR);

    // If contracts is set we're compiling every function in a 'contract' rather than just 'main'.
    if args.contracts {
        let compiled_contracts = try_vecmap(driver.get_all_contracts(), |contract| {
            compile_contract(&driver, contract, &args.compile_options)
        })?;

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
        let main = driver.main_function().map_err(|_| CliError::CompilationError)?;
        let program = compile_program(&driver, main, &args.compile_options, &args.circuit_name)?;
        save_and_preprocess_program(&program, &args.circuit_name, &circuit_dir)
    }
}

fn setup_driver(program_dir: &Path) -> Result<Driver, CliError> {
    let backend = crate::backends::ConcreteBackend;
    let mut driver = Resolver::resolve_root_config(program_dir, backend.np_language())?;
    add_std_lib(&mut driver);
    Ok(driver)
}

fn check_crate(program_dir: &Path, options: &CompileOptions) -> Result<Driver, CliError> {
    let mut driver = setup_driver(program_dir)?;
    driver.check_crate(options).map_err(|_| CliError::CompilationError)?;
    Ok(driver)
}

/// Compiles all of the functions associated with a Noir contract.
fn compile_contract(
    driver: &Driver,
    contract: Contract,
    compile_options: &CompileOptions,
) -> Result<CompiledContract, CliError> {
    let functions = try_btree_map(&contract.functions, |function| {
        let function_name = driver.function_name(*function).to_owned();
        let program_id = format!("{}-{}", contract.name, function_name);

        compile_program(driver, *function, compile_options, &program_id)
            .map(|program| (function_name, program))
    })?;

    Ok(CompiledContract { name: contract.name, functions })
}

fn compile_program(
    driver: &Driver,
    main: FuncId,
    compile_options: &CompileOptions,
    program_id: &str,
) -> Result<CompiledProgram, CliError> {
    driver
        .compile_no_check(compile_options, main)
        .map_err(|_| CliError::Generic(format!("'{}' failed to compile", program_id)))
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
) -> Result<noirc_driver::CompiledProgram, CliError> {
    let mut driver = setup_driver(program_dir)?;
    driver.compile_main(compile_options).map_err(|_| CliError::CompilationError)
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
