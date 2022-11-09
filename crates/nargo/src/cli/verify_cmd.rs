use super::compile_cmd::compile_circuit;
use super::{PROOFS_DIR, PROOF_EXT, VERIFIER_INPUT_FILE};
use crate::errors::{AbiError, CliError};
use acvm::{FieldElement, ProofSystemCompiler};
use clap::ArgMatches;
use noirc_abi::{input_parser::InputValue, Abi};
use noirc_driver::CompiledProgram;
use std::{collections::BTreeMap, path::Path, path::PathBuf};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let proof_name = args.subcommand_matches("verify").unwrap().value_of("proof").unwrap();
    let mut proof_path = std::path::PathBuf::new();
    proof_path.push(Path::new(PROOFS_DIR));

    proof_path.push(Path::new(proof_name));
    proof_path.set_extension(PROOF_EXT);

    let result = verify(proof_name)?;
    println!("Proof verified : {}\n", result);
    Ok(())
}

fn verify(proof_name: &str) -> Result<bool, CliError> {
    let curr_dir = std::env::current_dir().unwrap();
    let mut proof_path = PathBuf::new(); //or cur_dir?
    proof_path.push(PROOFS_DIR);
    proof_path.push(Path::new(proof_name));
    proof_path.set_extension(PROOF_EXT);
    verify_with_path(&curr_dir, &proof_path, false)
}

fn process_abi_with_verifier_input(
    abi: Abi,
    pi_map: BTreeMap<String, InputValue>,
) -> Result<Vec<FieldElement>, AbiError> {
    // Filter out any private inputs from the ABI.
    let public_abi = abi.public_abi();
    let num_pub_params = public_abi.num_parameters();

    let mut public_inputs = Vec::with_capacity(num_pub_params);

    for (param_name, param_type) in public_abi.parameters.clone().into_iter() {
        let value = pi_map
            .get(&param_name)
            .ok_or_else(|| AbiError::MissingParam(param_name.clone()))?
            .clone();

        if !value.matches_abi(&param_type) {
            return Err(AbiError::TypeMismatch {
                param_name,
                expected_type: param_type,
                actual_value: value,
            });
        }

        match value {
            InputValue::Field(elem) => public_inputs.push(elem),
            InputValue::Vec(vec_elem) => public_inputs.extend(vec_elem),
            InputValue::Undefined => return Err(AbiError::UndefinedInput(param_name)),
        }
    }

    // Check that no extra witness values have been provided.
    // Any missing values should be caught by the above for-loop so this only catches extra values.
    if num_pub_params != pi_map.len() {
        let param_names = public_abi.parameter_names();
        let unexpected_params: Vec<String> =
            pi_map
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

    Ok(public_inputs)
}

pub fn verify_with_path<P: AsRef<Path>>(
    program_dir: P,
    proof_path: P,
    show_ssa: bool,
) -> Result<bool, CliError> {
    let compiled_program = compile_circuit(program_dir.as_ref(), show_ssa)?;
    let mut public_inputs = BTreeMap::new();

    // Load public inputs (if any) from `VERIFIER_INPUT_FILE`.
    let public_abi = compiled_program.abi.clone().unwrap().public_abi();
    let num_pub_params = public_abi.num_parameters();
    if num_pub_params != 0 {
        let curr_dir = program_dir;
        public_inputs =
            noirc_abi::input_parser::Format::Toml.parse(curr_dir, VERIFIER_INPUT_FILE)?
    }

    let valid_proof = verify_proof(compiled_program, public_inputs, load_proof(proof_path)?)?;

    Ok(valid_proof)
}

fn verify_proof(
    compiled_program: CompiledProgram,
    public_inputs: BTreeMap<String, InputValue>,
    proof: Vec<u8>,
) -> Result<bool, CliError> {
    let public_inputs =
        process_abi_with_verifier_input(compiled_program.abi.unwrap(), public_inputs)?;

    let backend = crate::backends::ConcreteBackend;
    let valid_proof = backend.verify_from_cs(&proof, public_inputs, compiled_program.circuit);

    Ok(valid_proof)
}

fn load_proof<P: AsRef<Path>>(proof_path: P) -> Result<Vec<u8>, CliError> {
    let proof_hex: Vec<_> = std::fs::read(&proof_path)
        .map_err(|_| CliError::PathNotValid(proof_path.as_ref().to_path_buf()))?;
    let proof = hex::decode(proof_hex).map_err(CliError::ProofNotValid)?;

    Ok(proof)
}
