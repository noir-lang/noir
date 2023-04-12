use super::compile_cmd::compile_circuit;
use super::fs::{inputs::read_inputs_from_file, load_hex_data, program::read_program_from_file};
use super::NargoConfig;
use crate::{
    constants::{PROOFS_DIR, PROOF_EXT, TARGET_DIR, VERIFIER_INPUT_FILE},
    errors::CliError,
};

use clap::Args;
use nargo::artifacts::program::PreprocessedProgram;
use nargo::ops::preprocess_program;
use noirc_abi::input_parser::Format;
use noirc_driver::CompileOptions;
use std::path::Path;

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

    let valid_proof = verify_with_path(
        &config.program_dir,
        &proof_path,
        circuit_build_path.as_ref(),
        args.compile_options,
    )?;

    if valid_proof {
        Ok(())
    } else {
        Err(CliError::InvalidProof(proof_path))
    }
}

fn verify_with_path<P: AsRef<Path>>(
    program_dir: P,
    proof_path: P,
    circuit_build_path: Option<P>,
    compile_options: CompileOptions,
) -> Result<bool, CliError> {
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

    // Load public inputs (if any) from `VERIFIER_INPUT_FILE`.
    let public_abi = abi.public_abi();
    let (public_inputs_map, return_value) =
        read_inputs_from_file(program_dir, VERIFIER_INPUT_FILE, Format::Toml, &public_abi)?;

    let public_inputs = public_abi.encode(&public_inputs_map, return_value)?;
    let proof = load_hex_data(&proof_path)?;

    let valid_proof =
        nargo::ops::verify_proof(&backend, &bytecode, &proof, public_inputs, &verification_key)?;

    Ok(valid_proof)
}
