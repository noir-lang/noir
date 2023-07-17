use super::compile_cmd::compile_circuit;
use super::fs::{
    common_reference_string::{
        read_cached_common_reference_string, update_common_reference_string,
        write_cached_common_reference_string,
    },
    inputs::read_inputs_from_file,
    load_hex_data,
    program::read_program_from_file,
};
use super::NargoConfig;
use crate::{
    constants::{PROOFS_DIR, PROOF_EXT, TARGET_DIR, VERIFIER_INPUT_FILE},
    errors::CliError,
};

use acvm::Backend;
use clap::Args;
use nargo::artifacts::program::PreprocessedProgram;
use nargo::ops::{preprocess_program, verify_proof};
use noirc_abi::input_parser::Format;
use noirc_driver::CompileOptions;
use std::path::{Path, PathBuf};

/// Given a proof and a program, verify whether the proof is valid
#[derive(Debug, Clone, Args)]
pub(crate) struct VerifyCommand {
    /// The proof to verify
    proof: String,

    /// The name of the circuit build files (ACIR, proving and verification keys)
    circuit_name: Option<String>,

    /// The name of the toml file which contains the inputs for the verifier
    #[clap(long, short, default_value = VERIFIER_INPUT_FILE)]
    verifier_name: String,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: VerifyCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let proof_path =
        config.program_dir.join(PROOFS_DIR).join(&args.proof).with_extension(PROOF_EXT);

    let circuit_build_path = args
        .circuit_name
        .map(|circuit_name| config.program_dir.join(TARGET_DIR).join(circuit_name));

    verify_with_path(
        backend,
        &config.program_dir,
        proof_path,
        circuit_build_path.as_ref(),
        args.verifier_name,
        &args.compile_options,
    )
}

fn verify_with_path<B: Backend, P: AsRef<Path>>(
    backend: &B,
    program_dir: P,
    proof_path: PathBuf,
    circuit_build_path: Option<P>,
    verifier_name: String,
    compile_options: &CompileOptions,
) -> Result<(), CliError<B>> {
    let common_reference_string = read_cached_common_reference_string();

    let (common_reference_string, preprocessed_program) = match circuit_build_path {
        Some(circuit_build_path) => {
            let program = read_program_from_file(circuit_build_path)?;
            let common_reference_string = update_common_reference_string(
                backend,
                &common_reference_string,
                &program.bytecode,
            )
            .map_err(CliError::CommonReferenceStringError)?;
            (common_reference_string, program)
        }
        None => {
            let (program, _) =
                compile_circuit(backend, None, program_dir.as_ref(), compile_options)?;
            let common_reference_string =
                update_common_reference_string(backend, &common_reference_string, &program.circuit)
                    .map_err(CliError::CommonReferenceStringError)?;
            let (program, _) = preprocess_program(backend, true, &common_reference_string, program)
                .map_err(CliError::ProofSystemCompilerError)?;
            (common_reference_string, program)
        }
    };

    write_cached_common_reference_string(&common_reference_string);

    let PreprocessedProgram { abi, bytecode, verification_key, .. } = preprocessed_program;

    // Load public inputs (if any) from `verifier_name`.
    let public_abi = abi.public_abi();
    let (public_inputs_map, return_value) =
        read_inputs_from_file(program_dir, verifier_name.as_str(), Format::Toml, &public_abi)?;

    let public_inputs = public_abi.encode(&public_inputs_map, return_value)?;
    let proof = load_hex_data(&proof_path)?;

    let verification_key = verification_key
        .expect("Verification key should exist as `true` is passed to `preprocess_program`");
    let valid_proof = verify_proof(
        backend,
        &common_reference_string,
        &bytecode,
        &proof,
        public_inputs,
        &verification_key,
    )
    .map_err(CliError::ProofSystemCompilerError)?;

    if valid_proof {
        Ok(())
    } else {
        Err(CliError::InvalidProof(proof_path))
    }
}
