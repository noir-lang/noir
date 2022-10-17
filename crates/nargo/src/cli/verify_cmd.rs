use super::{PROOFS_DIR, PROOF_EXT, VERIFIER_INPUT_FILE};
use crate::{errors::CliError, resolver::Resolver};
use acvm::FieldElement;
use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use noirc_abi::{input_parser::InputValue, Abi};
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
) -> Result<Vec<FieldElement>, CliError> {
    let mut public_inputs = Vec::with_capacity(pi_map.len());

    for (param_name, param_type) in abi.parameters.into_iter() {
        let value = pi_map
            .get(&param_name)
            .unwrap_or_else(|| {
                panic!("ABI expects the parameter `{}`, but this was not found", param_name)
            })
            .clone();

        if !value.matches_abi(param_type) {
            return Err(CliError::Generic(format!("The parameters in the main do not match the parameters in the {}.toml file. \n Please check `{}` parameter. ", VERIFIER_INPUT_FILE,param_name)));
        }

        match value {
            InputValue::Field(elem) => public_inputs.push(elem),
            InputValue::Vec(vec_elem) => public_inputs.extend(vec_elem),
            InputValue::Undefined => {
                return Err(CliError::Generic(format!(
                    "The parameter {} is not defined in the {}.toml file.",
                    param_name, VERIFIER_INPUT_FILE
                )))
            }
        }
    }

    Ok(public_inputs)
}

pub fn verify_with_path<P: AsRef<Path>>(
    program_dir: P,
    proof_path: P,
    show_ssa: bool,
) -> Result<bool, CliError> {
    let driver = Resolver::resolve_root_config(program_dir.as_ref())?;
    let backend = crate::backends::ConcreteBackend;

    let compiled_program = driver.into_compiled_program(backend.np_language(), show_ssa);

    let public_abi = compiled_program.abi.clone().unwrap().public_abi();
    let num_pub_params = public_abi.num_parameters();
    let mut public_inputs = BTreeMap::new();
    if num_pub_params != 0 {
        let curr_dir = program_dir;
        public_inputs = noirc_abi::input_parser::Format::Toml
            .parse(curr_dir, VERIFIER_INPUT_FILE)
            .map_err(CliError::from)?;
    }

    if num_pub_params != public_inputs.len() {
        // return Err(CliError::Generic(format!("")));
        panic!(
            "Expected {} number of values in {}.toml, but got {} number of values",
            num_pub_params,
            VERIFIER_INPUT_FILE,
            public_inputs.len()
        )
    }

    let public_inputs = process_abi_with_verifier_input(public_abi, public_inputs)?;

    // XXX: Instead of unwrap, return a PathNotValidError
    let proof_hex: Vec<_> = std::fs::read(&proof_path).unwrap();
    // XXX: Instead of unwrap, return a ProofNotValidError
    let proof = hex::decode(proof_hex).unwrap();

    let valid_proof = backend.verify_from_cs(&proof, public_inputs, compiled_program.circuit);

    Ok(valid_proof)
}
