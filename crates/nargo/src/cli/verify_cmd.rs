use super::{
    compile_cmd::compile_circuit, dedup_public_input_indices_values, fetch_pk_and_vk,
    load_hex_data, read_inputs_from_file, InputMap,
};
use crate::{
    constants::{PROOFS_DIR, PROOF_EXT, TARGET_DIR, VERIFIER_INPUT_FILE},
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

    let circuit_name = args.value_of("circuit_name");

    let allow_warnings = args.is_present("allow-warnings");

    let circuit_build_path = if let Some(circuit_name) = circuit_name {
        let mut circuit_build_path = program_dir.clone();
        circuit_build_path.push(TARGET_DIR);
        circuit_build_path.push(circuit_name);
        Some(circuit_build_path)
    } else {
        None
    };

    let result =
        verify_with_path(program_dir, proof_path, circuit_build_path, false, allow_warnings)?;
    println!("Proof verified : {result}\n");
    Ok(())
}

pub fn verify_with_path<P: AsRef<Path>>(
    program_dir: P,
    proof_path: P,
    circuit_build_path: Option<P>,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<bool, CliError> {
    let compiled_program = compile_circuit(program_dir.as_ref(), show_ssa, allow_warnings)?;
    let (_, verification_key) =
        fetch_pk_and_vk(compiled_program.circuit.clone(), circuit_build_path, false, true)?;

    let mut public_inputs_map: InputMap = BTreeMap::new();

    // Load public inputs (if any) from `VERIFIER_INPUT_FILE`.
    let public_abi = compiled_program.abi.clone().unwrap().public_abi();
    let num_pub_params = public_abi.num_parameters();
    if num_pub_params != 0 {
        let current_dir = program_dir;
        public_inputs_map =
            read_inputs_from_file(current_dir, VERIFIER_INPUT_FILE, Format::Toml, public_abi)?;
    }

    let valid_proof = verify_proof(
        compiled_program,
        public_inputs_map,
        &load_hex_data(proof_path)?,
        verification_key,
    )?;

    Ok(valid_proof)
}

pub(crate) fn verify_proof(
    mut compiled_program: CompiledProgram,
    public_inputs_map: InputMap,
    proof: &[u8],
    verification_key: Vec<u8>,
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
    let valid_proof = backend.verify_with_vk(
        proof,
        dedup_public_values,
        compiled_program.circuit,
        verification_key,
    );

    Ok(valid_proof)
}
