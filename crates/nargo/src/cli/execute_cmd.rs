use std::path::Path;

use acvm::PartialWitnessGenerator;
use clap::Args;
use noirc_abi::input_parser::{Format, InputValue};
use noirc_abi::{InputMap, WitnessMap};
use noirc_driver::CompiledProgram;

use super::fs::{inputs::read_inputs_from_file, witness::save_witness_to_dir};
use super::NargoConfig;
use crate::{
    cli::compile_cmd::compile_circuit,
    constants::{PROVER_INPUT_FILE, TARGET_DIR},
    errors::CliError,
};

/// Executes a circuit to calculate its return value
#[derive(Debug, Clone, Args)]
pub(crate) struct ExecuteCommand {
    /// Write the execution witness to named file
    witness_name: Option<String>,

    /// Issue a warning for each unused variable instead of an error
    #[arg(short, long)]
    allow_warnings: bool,

    /// Emit debug information for the intermediate SSA IR
    #[arg(short, long)]
    show_ssa: bool,
}

pub(crate) fn run(args: ExecuteCommand, config: NargoConfig) -> Result<(), CliError> {
    let (return_value, solved_witness) =
        execute_with_path(&config.program_dir, args.show_ssa, args.allow_warnings)?;

    println!("Circuit witness successfully solved");
    if let Some(return_value) = return_value {
        println!("Circuit output: {return_value:?}");
    }
    if let Some(witness_name) = args.witness_name {
        let mut witness_dir = config.program_dir;
        witness_dir.push(TARGET_DIR);

        let witness_path = save_witness_to_dir(solved_witness, &witness_name, witness_dir)?;

        println!("Witness saved to {}", witness_path.display());
    }
    Ok(())
}

fn execute_with_path<P: AsRef<Path>>(
    program_dir: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<(Option<InputValue>, WitnessMap), CliError> {
    let compiled_program = compile_circuit(&program_dir, show_ssa, allow_warnings)?;

    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) = read_inputs_from_file(
        &program_dir,
        PROVER_INPUT_FILE,
        Format::Toml,
        &compiled_program.abi,
    )?;

    let solved_witness = execute_program(&compiled_program, &inputs_map)?;

    let public_abi = compiled_program.abi.public_abi();
    let (_, return_value) = public_abi.decode(&solved_witness)?;

    Ok((return_value, solved_witness))
}

pub(crate) fn execute_program(
    compiled_program: &CompiledProgram,
    inputs_map: &InputMap,
) -> Result<WitnessMap, CliError> {
    let mut solved_witness = compiled_program.abi.encode(inputs_map, None)?;

    let backend = crate::backends::ConcreteBackend;
    backend.solve(&mut solved_witness, compiled_program.circuit.opcodes.clone())?;

    Ok(solved_witness)
}
