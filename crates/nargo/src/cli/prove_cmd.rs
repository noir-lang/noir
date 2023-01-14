use clap::ArgMatches;
use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
};

use acvm::acir::{circuit::PublicInputs, native_types::Witness, FieldElement};
use acvm::{GateResolution, PartialWitnessGenerator, ProofSystemCompiler};
use noirc_abi::{
    errors::AbiError,
    input_parser::{Format, InputValue},
};

use super::{
    create_named_dir, read_inputs_from_file, write_inputs_to_file, write_to_file, PROOFS_DIR,
    PROOF_EXT, PROVER_INPUT_FILE, VERIFIER_INPUT_FILE,
};
use crate::errors::CliError;

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("prove").unwrap();
    let proof_name = args.value_of("proof_name").unwrap();
    let show_ssa = args.is_present("show-ssa");
    let allow_warnings = args.is_present("allow-warnings");
    prove(proof_name, show_ssa, allow_warnings)
}

/// In Barretenberg, the proof system adds a zero witness in the first index,
/// So when we add witness values, their index start from 1.
const WITNESS_OFFSET: u32 = 1;

fn prove(proof_name: &str, show_ssa: bool, allow_warnings: bool) -> Result<(), CliError> {
    let curr_dir = std::env::current_dir().unwrap();
    let mut proof_path = PathBuf::new();
    proof_path.push(PROOFS_DIR);
    let result = prove_with_path(proof_name, curr_dir, proof_path, show_ssa, allow_warnings);
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn compile_circuit_and_witness<P: AsRef<Path>>(
    program_dir: P,
    show_ssa: bool,
    allow_unused_variables: bool,
) -> Result<(noirc_driver::CompiledProgram, BTreeMap<Witness, FieldElement>), CliError> {
    let compiled_program = super::compile_cmd::compile_circuit(
        program_dir.as_ref(),
        show_ssa,
        allow_unused_variables,
    )?;
    let solved_witness = parse_and_solve_witness(program_dir, &compiled_program)?;
    Ok((compiled_program, solved_witness))
}

pub fn parse_and_solve_witness<P: AsRef<Path>>(
    program_dir: P,
    compiled_program: &noirc_driver::CompiledProgram,
) -> Result<BTreeMap<Witness, FieldElement>, CliError> {
    // Parse the initial witness values from Prover.toml
    let witness_map = read_inputs_from_file(&program_dir, PROVER_INPUT_FILE, Format::Toml)?;

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

    let public_abi = compiled_program.abi.as_ref().unwrap().clone().public_abi();
    let public_inputs = public_abi.decode(&encoded_public_inputs)?;

    // Write public inputs into Verifier.toml
    write_inputs_to_file(&public_inputs, &program_dir, VERIFIER_INPUT_FILE, Format::Toml)?;

    Ok(solved_witness)
}

fn solve_witness(
    compiled_program: &noirc_driver::CompiledProgram,
    witness_map: &BTreeMap<String, InputValue>,
) -> Result<BTreeMap<Witness, FieldElement>, CliError> {
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
    let solver_res = backend.solve(&mut solved_witness, compiled_program.circuit.gates.clone());

    match solver_res {
        GateResolution::UnsupportedOpcode(opcode) => return Err(CliError::Generic(format!(
                "backend does not currently support the {opcode} opcode. ACVM does not currently fall back to arithmetic gates.",
        ))),
        GateResolution::UnsatisfiedConstrain => return Err(CliError::Generic(
                "could not satisfy all constraints".to_string()
        )),
        GateResolution::Resolved => (),
        _ => unreachable!(),
    }

    Ok(solved_witness)
}

pub fn prove_with_path<P: AsRef<Path>>(
    proof_name: &str,
    program_dir: P,
    proof_dir: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<PathBuf, CliError> {
    let (mut compiled_program, solved_witness) =
        compile_circuit_and_witness(program_dir, show_ssa, allow_warnings)?;

    let backend = crate::backends::ConcreteBackend;

    // Since the public outputs are added into the public inputs list
    // There can be duplicates. We keep the duplicates for when one is
    // encoding the return values into the Verifier.toml
    // However, for creating a proof, we remove these duplicates.
    compiled_program.circuit.public_inputs =
        dedup_public_input_indices(compiled_program.circuit.public_inputs);

    let proof = backend.prove_with_meta(compiled_program.circuit, solved_witness);

    let mut proof_path = create_named_dir(proof_dir.as_ref(), "proof");
    proof_path.push(proof_name);
    proof_path.set_extension(PROOF_EXT);

    println!("proof : {}", hex::encode(&proof));

    let path = write_to_file(hex::encode(&proof).as_bytes(), &proof_path);
    println!("Proof successfully created and located at {path}");
    println!("{:?}", std::fs::canonicalize(&proof_path));

    Ok(proof_path)
}

// Removes duplicates from the list of public input witnesses
fn dedup_public_input_indices(indices: PublicInputs) -> PublicInputs {
    let duplicates_removed: HashSet<_> = indices.0.into_iter().collect();
    PublicInputs(duplicates_removed.into_iter().collect())
}

// Removes duplicates from the list of public input witnesses and the
// associated list of duplicate values.
pub(crate) fn dedup_public_input_indices_values(
    indices: PublicInputs,
    values: Vec<FieldElement>,
) -> (PublicInputs, Vec<FieldElement>) {
    // Assume that the public input index lists and the values contain duplicates
    assert_eq!(indices.0.len(), values.len());

    let mut public_inputs_without_duplicates = Vec::new();
    let mut already_seen_public_indices = HashSet::new();

    for (index, value) in indices.0.iter().zip(values) {
        if !already_seen_public_indices.contains(&index) {
            already_seen_public_indices.insert(index);
            public_inputs_without_duplicates.push(value)
        }
    }

    (
        PublicInputs(already_seen_public_indices.into_iter().cloned().collect()),
        public_inputs_without_duplicates,
    )
}
