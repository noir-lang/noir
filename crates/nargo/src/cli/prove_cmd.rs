use std::{collections::BTreeMap, path::PathBuf};

use acvm::acir::native_types::Witness;
use acvm::FieldElement;
use acvm::ProofSystemCompiler;
use acvm::{GateResolution, PartialWitnessGenerator};
use clap::ArgMatches;
use noirc_abi::AbiType;
use noirc_abi::{input_parser::InputValue, Abi};
use std::path::Path;

use crate::errors::{AbiError, CliError};

use super::{
    create_named_dir, write_to_file, PROOFS_DIR, PROOF_EXT, PROVER_INPUT_FILE, VERIFIER_INPUT_FILE,
};

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

/// Ordering is important here, which is why we need the ABI to tell us what order to add the elements in
/// We then need the witness map to get the elements field values.
fn process_abi_with_input(
    abi: Abi,
    witness_map: &BTreeMap<String, InputValue>,
) -> Result<Vec<FieldElement>, AbiError> {
    let num_params = abi.num_parameters();
    let mut encoded_inputs = Vec::new();

    let return_witness_len: u32 = abi
        .parameters
        .iter()
        .find(|x| x.0 == noirc_frontend::hir_def::function::MAIN_RETURN_NAME)
        .map_or(0, |(_, return_type)| return_type.field_count());

    for (param_name, param_type) in abi.parameters.clone().into_iter() {
        let value = witness_map
            .get(&param_name)
            .ok_or_else(|| AbiError::MissingParam(param_name.clone()))?
            .clone();

        if !value.matches_abi(&param_type) {
            return Err(AbiError::TypeMismatch { param_name, param_type, value });
        }

        if param_name != noirc_frontend::hir_def::function::MAIN_RETURN_NAME
            || return_witness_len != 1
            || !matches!(value, InputValue::Undefined)
        {
            let encoded_input = input_value_into_witness(value, param_name)?;
            encoded_inputs.extend(encoded_input);
        }
    }

    // Check that no extra witness values have been provided.
    // Any missing values should be caught by the above for-loop so this only catches extra values.
    if num_params != witness_map.len() {
        let param_names = abi.parameter_names();
        let unexpected_params: Vec<String> =
            witness_map.keys().filter(|param| !param_names.contains(param)).cloned().collect();
        return Err(AbiError::UnexpectedParams(unexpected_params));
    }

    Ok(encoded_inputs)
}

fn input_value_into_witness(
    value: InputValue,
    param_name: String,
) -> Result<Vec<FieldElement>, AbiError> {
    let mut encoded_value = Vec::new();
    match value {
        InputValue::Field(elem) => encoded_value.push(elem),
        InputValue::Vec(vec_elem) => encoded_value.extend(vec_elem),
        InputValue::Struct(object) => {
            for (name, value) in object {
                encoded_value.extend(input_value_into_witness(value, name)?)
            }
        }
        InputValue::Undefined => return Err(AbiError::UndefinedInput(param_name)),
    }
    Ok(encoded_value)
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
    let witness_map =
        noirc_abi::input_parser::Format::Toml.parse(&program_dir, PROVER_INPUT_FILE)?;

    // Solve the remaining witnesses
    let solved_witness = solve_witness(compiled_program, &witness_map)?;

    let abi = compiled_program.abi.as_ref().unwrap();

    // We allow the user to optionally not provide a value for the circuit's return value, so this may be missing from
    // `witness_map`. We must then decode these from the circuit's witness values.
    let inputs_vector: Vec<FieldElement> = (0..abi.field_count())
        .map(|index| solved_witness[&Witness::new(index + WITNESS_OFFSET)])
        .collect();
    let decoded_inputs = abi_decode(abi, &inputs_vector);

    // Write public inputs into Verifier.toml
    export_public_inputs(&decoded_inputs, abi.clone(), &program_dir)?;

    Ok(solved_witness)
}

fn solve_witness(
    compiled_program: &noirc_driver::CompiledProgram,
    witness_map: &BTreeMap<String, InputValue>,
) -> Result<BTreeMap<Witness, FieldElement>, CliError> {
    let abi = compiled_program.abi.as_ref().unwrap();
    let encoded_inputs =
        process_abi_with_input(abi.clone(), witness_map).map_err(|error| match error {
            AbiError::UndefinedInput(_) => {
                CliError::Generic(format!("{} in the {}.toml file.", error, PROVER_INPUT_FILE))
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
                "backend does not currently support the {} opcode. ACVM does not currently fall back to arithmetic gates.",
                opcode
        ))),
        GateResolution::UnsatisfiedConstrain => return Err(CliError::Generic(
                "could not satisfy all constraints".to_string()
        )),
        GateResolution::Resolved => (),
        _ => unreachable!(),
    }

    Ok(solved_witness)
}

fn abi_decode(abi: &Abi, encoded_inputs: &Vec<FieldElement>) -> BTreeMap<String, InputValue> {
    let mut index = 0;
    let mut decoded_inputs = BTreeMap::new();

    for (param_name, param_type) in &abi.parameters {
        let (next_index, decoded_value) =
            read_value_from_witness(index, &encoded_inputs, &param_type);

        decoded_inputs.insert(param_name.to_owned(), decoded_value);

        index = next_index;
    }
    decoded_inputs
}

fn read_value_from_witness(
    initial_index: usize,
    encoded_inputs: &Vec<FieldElement>,
    value_type: &AbiType,
) -> (usize, InputValue) {
    let mut index = initial_index;

    let value = match value_type {
        AbiType::Field(_) | AbiType::Integer { .. } => {
            let field_element = encoded_inputs[index];
            index += 1;

            InputValue::Field(field_element)
        }
        AbiType::Array { length, .. } => {
            let field_elements = &encoded_inputs[index..index + (*length as usize)];

            index += *length as usize;
            InputValue::Vec(field_elements.to_vec())
        }
        AbiType::Struct { fields, .. } => {
            let mut struct_map = BTreeMap::new();

            for (field_key, param_type) in fields {
                let (next_index, field_value) =
                    read_value_from_witness(index, encoded_inputs, param_type);

                struct_map.insert(field_key.to_owned(), field_value);
                index = next_index;
            }

            InputValue::Struct(struct_map)
        }
    };

    (index, value)
}

fn export_public_inputs<P: AsRef<Path>>(
    decoded_inputs: &BTreeMap<String, InputValue>,
    abi: Abi,
    path: P,
) -> Result<(), noirc_abi::errors::InputParserError> {
    let public_inputs = abi
        .public_abi()
        .parameter_names()
        .into_iter()
        .map(|param_name| (param_name.to_owned(), decoded_inputs[param_name].to_owned()))
        .collect();

    //serialise public inputs into verifier.toml
    noirc_abi::input_parser::Format::Toml.serialise(&path, VERIFIER_INPUT_FILE, &public_inputs)
}

pub fn prove_with_path<P: AsRef<Path>>(
    proof_name: &str,
    program_dir: P,
    proof_dir: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<PathBuf, CliError> {
    let (compiled_program, solved_witness) =
        compile_circuit_and_witness(program_dir, show_ssa, allow_warnings)?;

    let backend = crate::backends::ConcreteBackend;
    let proof = backend.prove_with_meta(compiled_program.circuit, solved_witness);

    let mut proof_path = create_named_dir(proof_dir.as_ref(), "proof");
    proof_path.push(proof_name);
    proof_path.set_extension(PROOF_EXT);

    println!("proof : {}", hex::encode(&proof));

    let path = write_to_file(hex::encode(&proof).as_bytes(), &proof_path);
    println!("Proof successfully created and located at {}", path);
    println!("{:?}", std::fs::canonicalize(&proof_path));

    Ok(proof_path)
}
