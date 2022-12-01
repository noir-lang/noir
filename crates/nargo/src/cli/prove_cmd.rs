use std::{collections::BTreeMap, path::PathBuf};

use acvm::acir::native_types::Witness;
use acvm::FieldElement;
use acvm::ProofSystemCompiler;
use acvm::{GateResolution, PartialWitnessGenerator};
use clap::ArgMatches;
use noirc_abi::AbiType;
use noirc_abi::{input_parser::InputValue, Abi};
use std::path::Path;

use crate::errors::CliError;

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
) -> Result<(BTreeMap<Witness, FieldElement>, Option<Witness>), CliError> {
    let mut solved_witness = BTreeMap::new();

    let mut index = 0;
    let mut return_witness = None;
    let return_witness_len: u32 = abi
        .parameters
        .iter()
        .find(|x| x.0 == noirc_frontend::hir_def::function::MAIN_RETURN_NAME)
        .map_or(0, |(_, return_type)| return_type.field_count());

    for (param_name, param_type) in abi.parameters.into_iter() {
        let value = witness_map
            .get(&param_name)
            .unwrap_or_else(|| {
                panic!("ABI expects the parameter `{}`, but this was not found", param_name)
            })
            .clone();

        if !value.matches_abi(param_type) {
            return Err(CliError::Generic(format!("The parameters in the main do not match the parameters in the {}.toml file. \n Please check `{}` parameter ", PROVER_INPUT_FILE,param_name)));
        }

        (index, return_witness) = input_value_into_witness(
            value,
            index,
            return_witness,
            &mut solved_witness,
            param_name,
            return_witness_len,
        )?;
    }
    Ok((solved_witness, return_witness))
}

fn input_value_into_witness(
    value: InputValue,
    initial_index: u32,
    initial_return_witness: Option<Witness>,
    solved_witness: &mut BTreeMap<Witness, FieldElement>,
    param_name: String,
    return_witness_len: u32,
) -> Result<(u32, Option<Witness>), CliError> {
    let mut index = initial_index;
    let mut return_witness = initial_return_witness;
    match value {
        InputValue::Field(element) => {
            let old_value = solved_witness.insert(Witness::new(index + WITNESS_OFFSET), element);
            assert!(old_value.is_none());
            index += 1;
        }
        InputValue::Vec(arr) => {
            for element in arr {
                let old_value =
                    solved_witness.insert(Witness::new(index + WITNESS_OFFSET), element);
                assert!(old_value.is_none());
                index += 1;
            }
        }
        InputValue::Struct(object) => {
            for (name, value) in object {
                (index, return_witness) = input_value_into_witness(
                    value,
                    index,
                    return_witness,
                    solved_witness,
                    name,
                    return_witness_len,
                )?;
            }
        }
        InputValue::Undefined => {
            assert_eq!(
                param_name,
                noirc_frontend::hir_def::function::MAIN_RETURN_NAME,
                "input value {} is not defined",
                param_name
            );
            return_witness = Some(Witness::new(index + WITNESS_OFFSET));

            //We do not support undefined arrays for now - TODO
            if return_witness_len != 1 {
                return Err(CliError::Generic(
                    "Values of array returned from main must be specified in prover toml file"
                        .to_string(),
                ));
            }
            index += return_witness_len;
            //XXX We do not support (yet) array of arrays
        }
    }
    Ok((index, return_witness))
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
    let solved_witness = solve_witness(program_dir, &compiled_program)?;
    Ok((compiled_program, solved_witness))
}

pub fn solve_witness<P: AsRef<Path>>(
    program_dir: P,
    compiled_program: &noirc_driver::CompiledProgram,
) -> Result<BTreeMap<Witness, FieldElement>, CliError> {
    // Parse the initial witness values
    let witness_map = noirc_abi::input_parser::Format::Toml
        .parse(&program_dir, PROVER_INPUT_FILE)
        .map_err(CliError::from)?;

    // Check that enough witness values were supplied
    let num_params = compiled_program.abi.as_ref().unwrap().num_parameters();
    if num_params != witness_map.len() {
        panic!(
            "Expected {} number of values, but got {} number of values",
            num_params,
            witness_map.len()
        )
    }
    // Map initial witnesses with their values
    let abi = compiled_program.abi.as_ref().unwrap();
    // Solve the remaining witnesses
    let (mut solved_witness, rv) = process_abi_with_input(abi.clone(), &witness_map)?;

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

    // (over)writes verifier.toml
    export_public_inputs(rv, &solved_witness, &witness_map, abi, &program_dir)
        .map_err(CliError::from)?;

    Ok(solved_witness)
}

fn export_public_inputs<P: AsRef<Path>>(
    w_ret: Option<Witness>,
    solved_witness: &BTreeMap<Witness, FieldElement>,
    witness_map: &BTreeMap<String, InputValue>,
    abi: &Abi,
    path: P,
) -> Result<(), noirc_abi::errors::InputParserError> {
    // generate a name->value map for the public inputs, using the ABI and witness_map:
    let mut public_inputs = BTreeMap::new();
    for i in &abi.parameters {
        if i.1.is_public() {
            let v = &witness_map[&i.0];

            let iv = if matches!(*v, InputValue::Undefined) {
                let w_ret = w_ret.unwrap();
                match &i.1 {
                    AbiType::Array { length, .. } => {
                        let return_values = noirc_frontend::util::vecmap(0..*length, |i| {
                            *solved_witness.get(&Witness::new(w_ret.0 + i as u32)).unwrap()
                        });
                        InputValue::Vec(return_values)
                    }
                    _ => InputValue::Field(*solved_witness.get(&w_ret).unwrap()),
                }
            } else {
                v.clone()
            };
            public_inputs.insert(i.0.clone(), iv);
        }
    }
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
