use super::{
    compile_cmd::compile_circuit, dedup_public_input_indices_values, fetch_pk_and_vk,
    load_hex_data, read_inputs_from_file, InputMap, NargoConfig,
};
use crate::{
    constants::{PROOFS_DIR, PROOF_EXT, TARGET_DIR, VERIFIER_INPUT_FILE},
    errors::CliError,
};
use acvm::ProofSystemCompiler;
use clap::Args;
use noirc_abi::errors::AbiError;
use noirc_abi::input_parser::Format;
use noirc_driver::CompiledProgram;
use std::{collections::BTreeMap, path::Path};

/// Given a proof and a program, verify whether the proof is valid
#[derive(Debug, Clone, Args)]
pub(crate) struct VerifyCommand {
    /// The proof to verify
    proof: String,

    /// The name of the circuit build files (ACIR, proving and verification keys)
    circuit_name: Option<String>,

    /// Issue a warning for each unused variable instead of an error
    #[arg(short, long)]
    allow_warnings: bool,
}

pub(crate) fn run(args: VerifyCommand, config: NargoConfig) -> Result<(), CliError> {
    let mut proof_path = config.program_dir.clone();
    proof_path.push(Path::new(PROOFS_DIR));
    proof_path.push(Path::new(&args.proof));
    proof_path.set_extension(PROOF_EXT);

    let circuit_build_path = if let Some(circuit_name) = args.circuit_name {
        let mut circuit_build_path = config.program_dir.clone();
        circuit_build_path.push(TARGET_DIR);
        circuit_build_path.push(circuit_name);
        Some(circuit_build_path)
    } else {
        None
    };

    let result = verify_with_path(
        config.program_dir,
        proof_path,
        circuit_build_path,
        false,
        args.allow_warnings,
    )?;
    println!("Proof verified : {result}");

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
    let public_abi = compiled_program.abi.clone().public_abi();
    let num_pub_params = public_abi.num_parameters();
    if num_pub_params != 0 {
        let current_dir = program_dir;
        public_inputs_map =
            read_inputs_from_file(current_dir, VERIFIER_INPUT_FILE, Format::Toml, &public_abi)?;
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
    let public_abi = compiled_program.abi.public_abi();
    let public_inputs =
        public_abi.encode_to_array(&public_inputs_map).map_err(|error| match error {
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
