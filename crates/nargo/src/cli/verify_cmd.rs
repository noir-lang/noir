use super::{
    compile_cmd::compile_circuit, dedup_public_input_indices_values, read_inputs_from_file,
    InputMap,
};
use crate::{
    constants::{PROOFS_DIR, PROOF_EXT, VERIFIER_INPUT_FILE},
    errors::CliError,
};
use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use noirc_abi::errors::AbiError;
use noirc_abi::input_parser::Format;
use noirc_driver::CompiledProgram;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("verify").unwrap();

    let proof_name = args.value_of("proof").unwrap();
    let program_dir =
        args.value_of("path").map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);

    let mut proof_path = program_dir.clone();
    proof_path.push(Path::new(PROOFS_DIR));
    proof_path.push(Path::new(proof_name));
    proof_path.set_extension(PROOF_EXT);

    let allow_warnings = args.is_present("allow-warnings");
    let result = verify_with_path(program_dir, proof_path, false, allow_warnings)?;
    println!("Proof verified : {result}\n");
    Ok(())
}

pub fn verify_with_path<P: AsRef<Path>>(
    program_dir: P,
    proof_path: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<bool, CliError> {
    let compiled_program = compile_circuit(program_dir.as_ref(), show_ssa, allow_warnings)?;
    let mut public_inputs_map: InputMap = BTreeMap::new();

    // Load public inputs (if any) from `VERIFIER_INPUT_FILE`.
    let public_abi = compiled_program.abi.clone().unwrap().public_abi();
    let num_pub_params = public_abi.num_parameters();
    if num_pub_params != 0 {
        let current_dir = program_dir;
        public_inputs_map =
            read_inputs_from_file(current_dir, VERIFIER_INPUT_FILE, Format::Toml, public_abi)?;
    }

    let valid_proof = verify_proof(compiled_program, public_inputs_map, &load_proof(proof_path)?)?;

    Ok(valid_proof)
}

pub(crate) fn verify_proof(
    mut compiled_program: CompiledProgram,
    public_inputs_map: InputMap,
    proof: &[u8],
) -> Result<bool, CliError> {
    let public_abi = compiled_program.abi.unwrap().public_abi();
    let public_inputs =
        public_abi.encode(&public_inputs_map, false).map_err(|error| match error {
            AbiError::UndefinedInput(_) => {
                CliError::Generic(format!("{error} in the {VERIFIER_INPUT_FILE}.toml file."))
            }
            _ => CliError::from(error),
        })?;

    // Similarly to when proving -- we must remove the duplicate public witnesses which
    // can be present because a public input can also be added as a public output.
    let (dedup_public_indices, dedup_public_values) =
        dedup_public_input_indices_values(compiled_program.circuit.public_inputs, public_inputs);
    compiled_program.circuit.public_inputs = dedup_public_indices;

    let backend = crate::backends::ConcreteBackend;
    let valid_proof = backend.verify_from_cs(proof, dedup_public_values, compiled_program.circuit);

    Ok(valid_proof)
}

fn load_proof<P: AsRef<Path>>(proof_path: P) -> Result<Vec<u8>, CliError> {
    let proof_hex: Vec<_> = std::fs::read(&proof_path)
        .map_err(|_| CliError::PathNotValid(proof_path.as_ref().to_path_buf()))?;
    let proof = hex::decode(proof_hex).map_err(CliError::ProofNotValid)?;

    Ok(proof)
}
