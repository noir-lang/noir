use super::fs::{
    inputs::read_inputs_from_file, keys::fetch_pk_and_vk, load_hex_data,
    program::read_program_from_file,
};
use super::{compile_cmd::compile_circuit, InputMap, NargoConfig};
use crate::{
    constants::{PROOFS_DIR, PROOF_EXT, TARGET_DIR, VERIFIER_INPUT_FILE},
    errors::CliError,
};
use acvm::ProofSystemCompiler;
use clap::Args;
use noirc_abi::input_parser::{Format, InputValue};
use noirc_driver::{CompileOptions, CompiledProgram};
use std::path::{Path, PathBuf};

/// Given a proof and a program, verify whether the proof is valid
#[derive(Debug, Clone, Args)]
pub(crate) struct VerifyCommand {
    /// The proof to verify
    proof: String,

    /// The name of the circuit build files (ACIR, proving and verification keys)
    circuit_name: Option<String>,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(args: VerifyCommand, config: NargoConfig) -> Result<(), CliError> {
    let proof_path =
        config.program_dir.join(PROOFS_DIR).join(&args.proof).with_extension(PROOF_EXT);

    let circuit_build_path = args
        .circuit_name
        .map(|circuit_name| config.program_dir.join(TARGET_DIR).join(circuit_name));

    verify_with_path(config.program_dir, proof_path, circuit_build_path, args.compile_options)
}

fn verify_with_path<P: AsRef<Path>>(
    program_dir: P,
    proof_path: PathBuf,
    circuit_build_path: Option<P>,
    compile_options: CompileOptions,
) -> Result<(), CliError> {
    let (compiled_program, verification_key) = match circuit_build_path {
        Some(circuit_build_path) => {
            let compiled_program = read_program_from_file(&circuit_build_path)?;

            let (_, verification_key) =
                fetch_pk_and_vk(&compiled_program.circuit, circuit_build_path, false, true)?;
            (compiled_program, verification_key)
        }
        None => {
            let compiled_program = compile_circuit(program_dir.as_ref(), &compile_options)?;

            let backend = crate::backends::ConcreteBackend;
            let (_, verification_key) = backend.preprocess(&compiled_program.circuit);
            (compiled_program, verification_key)
        }
    };

    // Load public inputs (if any) from `VERIFIER_INPUT_FILE`.
    let public_abi = compiled_program.abi.clone().public_abi();
    let (public_inputs_map, return_value) =
        read_inputs_from_file(program_dir, VERIFIER_INPUT_FILE, Format::Toml, &public_abi)?;

    verify_proof(
        &compiled_program,
        public_inputs_map,
        return_value,
        &load_hex_data(&proof_path)?,
        &verification_key,
        proof_path,
    )
}

pub(crate) fn verify_proof(
    compiled_program: &CompiledProgram,
    public_inputs_map: InputMap,
    return_value: Option<InputValue>,
    proof: &[u8],
    verification_key: &[u8],
    proof_name: PathBuf,
) -> Result<(), CliError> {
    let public_abi = compiled_program.abi.clone().public_abi();
    let public_inputs = public_abi.encode(&public_inputs_map, return_value)?;

    let backend = crate::backends::ConcreteBackend;
    let valid_proof =
        backend.verify_with_vk(proof, public_inputs, &compiled_program.circuit, verification_key);

    if valid_proof {
        Ok(())
    } else {
        Err(CliError::InvalidProof(proof_name))
    }
}
