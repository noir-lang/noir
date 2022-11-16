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
    prove(proof_name, show_ssa)
}

/// In Barretenberg, the proof system adds a zero witness in the first index,
/// So when we add witness values, their index start from 1.
const WITNESS_OFFSET: u32 = 1;

fn prove(proof_name: &str, show_ssa: bool) -> Result<(), CliError> {
    let curr_dir = std::env::current_dir().unwrap();
    let mut proof_path = PathBuf::new();
    proof_path.push(PROOFS_DIR);
    let result = prove_with_path(proof_name, curr_dir, proof_path, show_ssa);
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
) -> Result<(BTreeMap<Witness, FieldElement>, Option<Witness>), AbiError> {
    let num_params = abi.num_parameters();
    let mut solved_witness = BTreeMap::new();

    let mut index = 0;
    let mut return_witness = None;
    let return_witness_len = if let Some(return_param) =
        abi.parameters.iter().find(|x| x.0 == noirc_frontend::hir_def::function::MAIN_RETURN_NAME)
    {
        match &return_param.1 {
            AbiType::Array { length, .. } => *length as u32,
            AbiType::Integer { .. } | AbiType::Field(_) => 1,
        }
    } else {
        0
    };
    for (param_name, param_type) in abi.parameters.clone().into_iter() {
        let value = witness_map
            .get(&param_name)
            .ok_or_else(|| AbiError::MissingParam(param_name.clone()))?
            .clone();

        if !value.matches_abi(&param_type) {
            return Err(AbiError::TypeMismatch { param_name, param_type, value });
        }

        match value {
            InputValue::Field(element) => {
                let old_value =
                    solved_witness.insert(Witness::new(index + WITNESS_OFFSET), element);
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
            InputValue::Undefined => {
                if param_name != noirc_frontend::hir_def::function::MAIN_RETURN_NAME {
                    return Err(AbiError::UndefinedInput(param_name));
                }
                return_witness = Some(Witness::new(index + WITNESS_OFFSET));

                //We do not support undefined arrays for now - TODO
                if return_witness_len != 1 {
                    return Err(AbiError::Generic(
                        "Values of array returned from main must be specified".to_string(),
                    ));
                }
                index += return_witness_len;
                //XXX We do not support (yet) array of arrays
            }
        }
    }

    // Check that no extra witness values have been provided.
    // Any missing values should be caught by the above for-loop so this only catches extra values.
    if num_params != witness_map.len() {
        let param_names = abi.parameter_names();
        let unexpected_params: Vec<String> =
            witness_map
                .keys()
                .filter_map(|param| {
                    if param_names.contains(&param) {
                        None
                    } else {
                        Some(param.to_owned())
                    }
                })
                .collect();
        return Err(AbiError::UnexpectedParams(unexpected_params));
    }

    Ok((solved_witness, return_witness))
}

pub fn compile_circuit_and_witness<P: AsRef<Path>>(
    program_dir: P,
    show_ssa: bool,
) -> Result<(noirc_driver::CompiledProgram, BTreeMap<Witness, FieldElement>), CliError> {
    let compiled_program = super::compile_cmd::compile_circuit(program_dir.as_ref(), show_ssa)?;
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
    let (solved_witness, return_value) = solve_witness(compiled_program, &witness_map)?;

    // Write public inputs into Verifier.toml
    let abi = compiled_program.abi.as_ref().unwrap();
    export_public_inputs(return_value, &solved_witness, &witness_map, abi, &program_dir)?;

    Ok(solved_witness)
}

fn solve_witness(
    compiled_program: &noirc_driver::CompiledProgram,
    witness_map: &BTreeMap<String, InputValue>,
) -> Result<(BTreeMap<Witness, FieldElement>, Option<Witness>), CliError> {
    let abi = compiled_program.abi.as_ref().unwrap();
    let (mut solved_witness, return_value) = process_abi_with_input(abi.clone(), witness_map)
        .map_err(|error| match error {
            AbiError::UndefinedInput(_) => {
                CliError::Generic(format!("{} in the {}.toml file.", error, PROVER_INPUT_FILE))
            }
            _ => CliError::from(error),
        })?;

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

    Ok((solved_witness, return_value))
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
) -> Result<PathBuf, CliError> {
    let (compiled_program, solved_witness) = compile_circuit_and_witness(program_dir, show_ssa)?;

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
