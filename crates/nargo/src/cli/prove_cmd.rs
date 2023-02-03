use std::path::{Path, PathBuf};

use acvm::acir::native_types::Witness;
use acvm::FieldElement;
use acvm::PartialWitnessGenerator;
use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use noirc_abi::errors::AbiError;
use noirc_abi::input_parser::Format;
use noirc_abi::Abi;

use super::{create_named_dir, read_inputs_from_file, write_inputs_to_file, write_to_file};
use super::{InputMap, WitnessMap};
use crate::cli::dedup_public_input_indices;
use crate::{
    constants::{PROOFS_DIR, PROOF_EXT, PROVER_INPUT_FILE, VERIFIER_INPUT_FILE},
    errors::CliError,
};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("prove").unwrap();
    let proof_name = args.value_of("proof_name");
    let show_ssa = args.is_present("show-ssa");
    let allow_warnings = args.is_present("allow-warnings");

    prove(proof_name, show_ssa, allow_warnings)
}

/// In Barretenberg, the proof system adds a zero witness in the first index,
/// So when we add witness values, their index start from 1.
const WITNESS_OFFSET: u32 = 1;

fn prove(proof_name: Option<&str>, show_ssa: bool, allow_warnings: bool) -> Result<(), CliError> {
    let current_dir = std::env::current_dir().unwrap();

    let mut proof_dir = PathBuf::new();
    proof_dir.push(PROOFS_DIR);

    prove_with_path(proof_name, current_dir, proof_dir, show_ssa, allow_warnings)?;

    Ok(())
}

pub fn prove_with_path<P: AsRef<Path>>(
    proof_name: Option<&str>,
    program_dir: P,
    proof_dir: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<Option<PathBuf>, CliError> {
    let (compiled_program, solved_witness) =
        compile_circuit_and_witness(program_dir, show_ssa, allow_warnings)?;

    let backend = crate::backends::ConcreteBackend;
    let proof = backend.prove_with_meta(compiled_program.circuit, solved_witness);

    println!("Proof successfully created");
    if let Some(proof_name) = proof_name {
        let proof_path = save_proof_to_dir(proof, proof_name, proof_dir)?;

        println!("Proof saved to {}", proof_path.display());
        Ok(Some(proof_path))
    } else {
        println!("{}", hex::encode(&proof));
        Ok(None)
    }
}

pub fn compile_circuit_and_witness<P: AsRef<Path>>(
    program_dir: P,
    show_ssa: bool,
    allow_unused_variables: bool,
) -> Result<(noirc_driver::CompiledProgram, WitnessMap), CliError> {
    let mut compiled_program = super::compile_cmd::compile_circuit(
        program_dir.as_ref(),
        show_ssa,
        allow_unused_variables,
    )?;
    let solved_witness = parse_and_solve_witness(program_dir, &compiled_program)?;

    // Since the public outputs are added into the public inputs list
    // There can be duplicates. We keep the duplicates for when one is
    // encoding the return values into the Verifier.toml
    // However, for creating a proof, we remove these duplicates.
    compiled_program.circuit.public_inputs =
        dedup_public_input_indices(compiled_program.circuit.public_inputs);

    Ok((compiled_program, solved_witness))
}

pub fn parse_and_solve_witness<P: AsRef<Path>>(
    program_dir: P,
    compiled_program: &noirc_driver::CompiledProgram,
) -> Result<WitnessMap, CliError> {
    let abi = compiled_program.abi.as_ref().expect("compiled program is missing an abi object");
    // Parse the initial witness values from Prover.toml
    let witness_map =
        read_inputs_from_file(&program_dir, PROVER_INPUT_FILE, Format::Toml, abi.clone())?;

    // Solve the remaining witnesses
    let solved_witness = solve_witness(compiled_program, &witness_map)?;

    // We allow the user to optionally not provide a value for the circuit's return value, so this may be missing from
    // `witness_map`. We must then decode these from the circuit's witness values.
    let encoded_public_inputs: Vec<FieldElement> = compiled_program
        .circuit
        .public_inputs
        .0
        .iter()
        .map(|index| solved_witness[index])
        .collect();

    let public_abi = abi.clone().public_abi();
    let public_inputs = public_abi.decode(&encoded_public_inputs)?;

    // Write public inputs into Verifier.toml
    write_inputs_to_file(&public_inputs, &program_dir, VERIFIER_INPUT_FILE, Format::Toml)?;

    Ok(solved_witness)
}

fn solve_witness(
    compiled_program: &noirc_driver::CompiledProgram,
    input_map: &InputMap,
) -> Result<WitnessMap, CliError> {
    let abi = compiled_program.abi.as_ref().unwrap().clone();
    let mut solved_witness =
        input_map_to_witness_map(abi, input_map).map_err(|error| match error {
            AbiError::UndefinedInput(_) => {
                CliError::Generic(format!("{error} in the {VERIFIER_INPUT_FILE}.toml file."))
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

fn save_proof_to_dir<P: AsRef<Path>>(
    proof: Vec<u8>,
    proof_name: &str,
    proof_dir: P,
) -> Result<PathBuf, CliError> {
    let mut proof_path = create_named_dir(proof_dir.as_ref(), "proof");
    proof_path.push(proof_name);
    proof_path.set_extension(PROOF_EXT);

    write_to_file(hex::encode(proof).as_bytes(), &proof_path);

    Ok(proof_path)
}
