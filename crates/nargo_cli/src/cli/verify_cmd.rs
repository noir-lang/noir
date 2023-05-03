use super::compile_cmd::compile_circuit;
use super::fs::{inputs::read_inputs_from_file, load_hex_data, program::read_program_from_file};
use super::{InputMap, NargoConfig};
use crate::{
    constants::{PROOFS_DIR, PROOF_EXT, TARGET_DIR, VERIFIER_INPUT_FILE},
    errors::CliError,
};
use acvm::ProofSystemCompiler;
use clap::Args;
use nargo::artifacts::program::PreprocessedProgram;
use nargo::ops::preprocess_program;
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
    let backend = crate::backends::ConcreteBackend::default();

    let preprocessed_program = match circuit_build_path {
        Some(circuit_build_path) => read_program_from_file(circuit_build_path)?,
        None => {
            let compiled_program =
                compile_circuit(&backend, program_dir.as_ref(), &compile_options)?;
            preprocess_program(&backend, compiled_program)?
        }
    };

    let PreprocessedProgram { abi, bytecode, verification_key, .. } = preprocessed_program;
    let compiled_program = CompiledProgram { abi, circuit: bytecode };

    // Load public inputs (if any) from `VERIFIER_INPUT_FILE`.
    let public_abi = compiled_program.abi.clone().public_abi();
    let (public_inputs_map, return_value) =
        read_inputs_from_file(program_dir, VERIFIER_INPUT_FILE, Format::Toml, &public_abi)?;

    verify_proof(
        &backend,
        &compiled_program,
        public_inputs_map,
        return_value,
        &load_hex_data(&proof_path)?,
        &verification_key,
        proof_path,
    )
}

pub(crate) fn verify_proof(
    backend: &impl ProofSystemCompiler,
    compiled_program: &CompiledProgram,
    public_inputs_map: InputMap,
    return_value: Option<InputValue>,
    proof: &[u8],
    verification_key: &[u8],
    proof_name: PathBuf,
) -> Result<(), CliError> {
    let public_abi = compiled_program.abi.clone().public_abi();
    let public_inputs = public_abi.encode(&public_inputs_map, return_value)?;

    let valid_proof = nargo::ops::verify_proof(
        backend,
        &compiled_program.circuit,
        proof,
        public_inputs,
        verification_key,
    )?;

    if valid_proof {
        Ok(())
    } else {
        Err(CliError::InvalidProof(proof_name))
    }
}
