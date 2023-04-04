use acvm::ProofSystemCompiler;
use noirc_driver::{CompileOptions, CompiledContract, CompiledProgram, Driver};
use std::path::Path;

use clap::Args;

use crate::resolver::DependencyResolutionError;
use crate::{constants::TARGET_DIR, errors::CliError, resolver::Resolver};

use super::fs::program::{save_contract_to_file, save_program_to_file};
use super::preprocess_cmd::{save_preprocess_data, PreprocessedData};
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
        let mut compiled_contracts = driver
            .compile_contracts(&args.compile_options)
            .map_err(|_| CliError::CompilationError)?;
        save_and_preprocess_contract(&mut compiled_contracts, &args.circuit_name, &circuit_dir)
    } else {
        let program = compile_circuit(&config.program_dir, &args.compile_options)?;
        save_and_preprocess_program(&program, &args.circuit_name, &circuit_dir)
    }
}

fn setup_driver(program_dir: &Path) -> Result<Driver, DependencyResolutionError> {
    let backend = nargo::backends::ConcreteBackend;
    Resolver::resolve_root_manifest(program_dir, backend.np_language())
}

/// Save a program to disk along with proving and verification keys.
fn save_and_preprocess_program(
    compiled_program: &CompiledProgram,
    circuit_name: &str,
    circuit_dir: &Path,
) -> Result<(), CliError> {
    save_program_to_file(compiled_program, circuit_name, circuit_dir);

    let preprocessed_data = PreprocessedData::from(&compiled_program.circuit);
    save_preprocess_data(&preprocessed_data, circuit_name, circuit_dir)?;
    Ok(())
}

/// Save a contract to disk along with proving and verification keys.
/// - The contract ABI is saved as one file, which contains all of the
/// functions defined in the contract.
/// - The proving and verification keys are namespaced since the file
/// could contain multiple contracts with the same name. The verification key is saved inside
/// of the ABI.
fn save_and_preprocess_contract(
    compiled_contracts: &mut [CompiledContract],
    circuit_name: &str,
    circuit_dir: &Path,
) -> Result<(), CliError> {
    for compiled_contract in compiled_contracts {
        // Preprocess all contract data
        // We are patching the verification key in our contract functions
        // so when we save it to disk, the ABI will have the verification key.
        let mut contract_preprocess_data = Vec::new();
        for contract_function in &mut compiled_contract.functions {
            let preprocessed_data = PreprocessedData::from(&contract_function.bytecode);
            contract_function.verification_key = Some(preprocessed_data.verification_key.clone());
            contract_preprocess_data.push(preprocessed_data);
        }

        // Unique identifier for a contract.
        let contract_id = format!("{}-{}", circuit_name, &compiled_contract.name);

        // Save contract ABI to file using the contract ID.
        // This includes the verification key.
        save_contract_to_file(compiled_contract, &contract_id, circuit_dir);

        // Save preprocessed data to disk
        //
        // TODO: This also includes the verification key, for now we save it in twice
        // TODO, once in ABI and once to disk as we did before.
        // TODO: A possible fix is to use optional fields in PreprocessedData
        // TODO struct. Then make VK None before saving so it is not saved to disk
        for (contract_function, preprocessed_data) in
            compiled_contract.functions.iter().zip(contract_preprocess_data)
        {
            // Create a name which uniquely identifies this contract function
            // over multiple contracts.
            let uniquely_identifying_program_name =
                format!("{}-{}", contract_id, contract_function.name);
            // Each program in a contract is preprocessed
            // Note: This can potentially be quite a long running process

            save_preprocess_data(
                &preprocessed_data,
                &uniquely_identifying_program_name,
                circuit_dir,
            )?;
        }
    }

    Ok(())
}

pub(crate) fn compile_circuit(
    program_dir: &Path,
    compile_options: &CompileOptions,
) -> Result<CompiledProgram, CliError> {
    let mut driver = setup_driver(program_dir)?;
    driver.compile_main(compile_options).map_err(|_| CliError::CompilationError)
}
