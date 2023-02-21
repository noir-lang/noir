use super::fs::{inputs::read_inputs_from_file, keys::fetch_pk_and_vk, load_hex_data};
use super::{compile_cmd::compile_circuit, InputMap, NargoConfig};
use crate::{
    constants::{PROOFS_DIR, PROOF_EXT, TARGET_DIR, VERIFIER_INPUT_FILE},
    errors::CliError,
};
use acvm::{FieldElement, ProofSystemCompiler};
use clap::Args;
use noirc_abi::input_parser::{Format, InputValue};
use noirc_driver::CompiledProgram;
use std::path::Path;

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
    let (_, verification_key) = match circuit_build_path {
        Some(circuit_build_path) => {
            fetch_pk_and_vk(&compiled_program.circuit, circuit_build_path, false, true)?
        }
        None => {
            let backend = crate::backends::ConcreteBackend;
            backend.preprocess(compiled_program.circuit.clone())
        }
    };

    // Load public inputs (if any) from `VERIFIER_INPUT_FILE`.
    let public_abi = compiled_program.abi.clone().public_abi();
    let (public_inputs_map, return_value) =
        read_inputs_from_file(program_dir, VERIFIER_INPUT_FILE, Format::Toml, &public_abi)?;

    let valid_proof = verify_proof(
        compiled_program,
        public_inputs_map,
        return_value,
        &load_hex_data(proof_path)?,
        verification_key,
    )?;

    Ok(valid_proof)
}

pub(crate) fn verify_proof(
    compiled_program: CompiledProgram,
    public_inputs_map: InputMap,
    return_value: Option<InputValue>,
    proof: &[u8],
    verification_key: Vec<u8>,
) -> Result<bool, CliError> {
    let public_abi = compiled_program.abi.public_abi();
    let public_inputs = public_abi.encode(&public_inputs_map, return_value)?;

    let public_inputs_vec: Vec<FieldElement> = public_inputs.values().copied().collect();

    let backend = crate::backends::ConcreteBackend;
    let valid_proof = backend.verify_with_vk(
        proof,
        public_inputs_vec,
        compiled_program.circuit,
        verification_key,
    );

    Ok(valid_proof)
}
