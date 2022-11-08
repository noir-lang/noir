use super::compile_cmd::compile_circuit;
use super::{PROOFS_DIR, PROOF_EXT, VERIFIER_INPUT_FILE};
use crate::errors::CliError;
use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use noirc_abi::encoding::process_abi_with_verifier_input;
use noirc_abi::errors::AbiError;
use noirc_abi::input_parser::InputValue;

use noirc_driver::CompiledProgram;
use std::{collections::BTreeMap, path::Path, path::PathBuf};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("verify").unwrap();

    let proof_name = args.value_of("proof").unwrap();
    let mut proof_path = std::path::PathBuf::new();
    proof_path.push(Path::new(PROOFS_DIR));

    proof_path.push(Path::new(proof_name));
    proof_path.set_extension(PROOF_EXT);

    let allow_warnings = args.is_present("allow-warnings");
    let result = verify(proof_name, allow_warnings)?;
    println!("Proof verified : {}\n", result);
    Ok(())
}

fn verify(proof_name: &str, allow_warnings: bool) -> Result<bool, CliError> {
    let curr_dir = std::env::current_dir().unwrap();
    let mut proof_path = PathBuf::new(); //or cur_dir?
    proof_path.push(PROOFS_DIR);
    proof_path.push(Path::new(proof_name));
    proof_path.set_extension(PROOF_EXT);
    verify_with_path(&curr_dir, &proof_path, false, allow_warnings)
}

pub fn verify_with_path<P: AsRef<Path>>(
    program_dir: P,
    proof_path: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<bool, CliError> {
    let compiled_program = compile_circuit(program_dir.as_ref(), show_ssa, allow_warnings)?;
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
    let public_inputs = process_abi_with_verifier_input(
        compiled_program.abi.unwrap(),
        public_inputs,
    )
    .map_err(|error| match error {
        AbiError::UndefinedInput(_) => {
            CliError::Generic(format!("{} in the {}.toml file.", error, VERIFIER_INPUT_FILE))
        }
        _ => CliError::from(error),
    })?;

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
