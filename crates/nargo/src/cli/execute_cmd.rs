use std::collections::BTreeMap;

use acvm::acir::native_types::Witness;
use acvm::FieldElement;
use acvm::PartialWitnessGenerator;
use clap::ArgMatches;
use noirc_abi::errors::AbiError;
use noirc_abi::input_parser::{Format, InputValue};
use noirc_abi::MAIN_RETURN_NAME;
use noirc_driver::CompiledProgram;
use std::path::Path;

use crate::{constants::PROVER_INPUT_FILE, errors::CliError};

use super::read_inputs_from_file;

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("execute").unwrap();
    let show_ssa = args.is_present("show-ssa");
    let allow_warnings = args.is_present("allow-warnings");
    let (return_value, _) = execute(show_ssa, allow_warnings)?;

    println!("Circuit witness successfully solved");
    if let Some(return_value) = return_value {
        println!("Circuit output: {return_value:?}");
    }
    Ok(())
}

/// In Barretenberg, the proof system adds a zero witness in the first index,
/// So when we add witness values, their index start from 1.
const WITNESS_OFFSET: u32 = 1;

fn execute(
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<(Option<InputValue>, BTreeMap<Witness, FieldElement>), CliError> {
    let curr_dir = std::env::current_dir().unwrap();

    let compiled_program =
        super::compile_cmd::compile_circuit(&curr_dir, show_ssa, allow_warnings)?;

    execute_program(curr_dir, &compiled_program)
}

pub(crate) fn execute_program<P: AsRef<Path>>(
    inputs_dir: P,
    compiled_program: &CompiledProgram,
) -> Result<(Option<InputValue>, BTreeMap<Witness, FieldElement>), CliError> {
    // Parse the initial witness values from Prover.toml
    let witness_map = read_inputs_from_file(
        inputs_dir,
        PROVER_INPUT_FILE,
        Format::Toml,
        compiled_program.abi.as_ref().unwrap().clone(),
    )?;

    // Solve the remaining witnesses
    let solved_witness = solve_witness(compiled_program, &witness_map)?;

    let public_inputs = extract_public_inputs(compiled_program, &solved_witness)?;
    let return_value = public_inputs.get(MAIN_RETURN_NAME).cloned();

    Ok((return_value, solved_witness))
}

pub(crate) fn extract_public_inputs(
    compiled_program: &noirc_driver::CompiledProgram,
    solved_witness: &BTreeMap<Witness, FieldElement>,
) -> Result<BTreeMap<String, InputValue>, AbiError> {
    let encoded_public_inputs: Vec<FieldElement> = compiled_program
        .circuit
        .public_inputs
        .0
        .iter()
        .map(|index| solved_witness[index])
        .collect();

    let public_abi = compiled_program.abi.as_ref().unwrap().clone().public_abi();

    public_abi.decode(&encoded_public_inputs)
}

pub(crate) fn solve_witness(
    compiled_program: &noirc_driver::CompiledProgram,
    witness_map: &BTreeMap<String, InputValue>,
) -> Result<BTreeMap<Witness, FieldElement>, CliError> {
    // Note that this currently accepts an input for the return value witness.
    // This doesn't really match with the expected usage of `nargo execute` as it's intended to calculate this.
    // See: https://github.com/noir-lang/noir/issues/624
    let abi = compiled_program.abi.as_ref().unwrap();
    let encoded_inputs = abi.clone().encode(witness_map, true).map_err(|error| match error {
        AbiError::UndefinedInput(_) => {
            CliError::Generic(format!("{error} in the {PROVER_INPUT_FILE}.toml file."))
        }
        _ => CliError::from(error),
    })?;

    let mut solved_witness: BTreeMap<Witness, FieldElement> = encoded_inputs
        .into_iter()
        .enumerate()
        .map(|(index, witness_value)| {
            let witness = Witness::new(WITNESS_OFFSET + (index as u32));
            (witness, witness_value)
        })
        .collect();

    let backend = crate::backends::ConcreteBackend;
    backend.solve(&mut solved_witness, compiled_program.circuit.opcodes.clone())?;

    Ok(solved_witness)
}
