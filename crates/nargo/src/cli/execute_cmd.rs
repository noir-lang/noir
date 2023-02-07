use std::path::{Path, PathBuf};

use acvm::acir::native_types::Witness;
use acvm::FieldElement;
use acvm::PartialWitnessGenerator;
use clap::ArgMatches;
use noirc_abi::errors::AbiError;
use noirc_abi::input_parser::{Format, InputValue};
use noirc_abi::{Abi, MAIN_RETURN_NAME};
use noirc_driver::CompiledProgram;

use super::{create_named_dir, read_inputs_from_file, write_to_file};
use super::{InputMap, WitnessMap};
use crate::{
    cli::compile_cmd::compile_circuit,
    constants::{PROVER_INPUT_FILE, TARGET_DIR, WITNESS_EXT},
    errors::CliError,
};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("execute").unwrap();
    let witness_name = args.value_of("witness_name");
    let show_ssa = args.is_present("show-ssa");
    let allow_warnings = args.is_present("allow-warnings");
    let program_dir =
        args.value_of("path").map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);

    let compiled_program = compile_circuit(&program_dir, show_ssa, allow_warnings)?;
    let (return_value, solved_witness) = execute_program(&program_dir, &compiled_program)?;

    println!("Circuit witness successfully solved");
    if let Some(return_value) = return_value {
        println!("Circuit output: {return_value:?}");
    }
    if let Some(witness_name) = witness_name {
        let mut witness_dir = program_dir;
        witness_dir.push(TARGET_DIR);

        let witness_path = save_witness_to_dir(solved_witness, witness_name, witness_dir)?;

        println!("Witness saved to {}", witness_path.display());
    }
    Ok(())
}

/// In Barretenberg, the proof system adds a zero witness in the first index,
/// So when we add witness values, their index start from 1.
const WITNESS_OFFSET: u32 = 1;

pub(crate) fn execute_program<P: AsRef<Path>>(
    inputs_dir: P,
    compiled_program: &CompiledProgram,
) -> Result<(Option<InputValue>, WitnessMap), CliError> {
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
    compiled_program: &CompiledProgram,
    solved_witness: &WitnessMap,
) -> Result<InputMap, AbiError> {
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
    compiled_program: &CompiledProgram,
    input_map: &InputMap,
) -> Result<WitnessMap, CliError> {
    let abi = compiled_program.abi.as_ref().unwrap().clone();
    let mut solved_witness =
        input_map_to_witness_map(abi, input_map).map_err(|error| match error {
            AbiError::UndefinedInput(_) => {
                CliError::Generic(format!("{error} in the {PROVER_INPUT_FILE}.toml file."))
            }
            _ => CliError::from(error),
        })?;

    let backend = crate::backends::ConcreteBackend;
    backend.solve(&mut solved_witness, compiled_program.circuit.opcodes.clone())?;

    Ok(solved_witness)
}

/// Given an InputMap and an Abi, produce a WitnessMap
///
/// In particular, this method shows one how to associate values in a Toml/JSON
/// file with witness indices
fn input_map_to_witness_map(abi: Abi, input_map: &InputMap) -> Result<WitnessMap, AbiError> {
    // The ABI map is first encoded as a vector of field elements
    let encoded_inputs = abi.encode(input_map, true)?;

    Ok(encoded_inputs
        .into_iter()
        .enumerate()
        .map(|(index, witness_value)| {
            let witness = Witness::new(WITNESS_OFFSET + (index as u32));
            (witness, witness_value)
        })
        .collect())
}

pub(crate) fn save_witness_to_dir<P: AsRef<Path>>(
    witness: WitnessMap,
    witness_name: &str,
    witness_dir: P,
) -> Result<PathBuf, CliError> {
    let mut witness_path = create_named_dir(witness_dir.as_ref(), "witness");
    witness_path.push(witness_name);
    witness_path.set_extension(WITNESS_EXT);

    let buf = Witness::to_bytes(&witness);

    write_to_file(buf.as_slice(), &witness_path);

    Ok(witness_path)
}
