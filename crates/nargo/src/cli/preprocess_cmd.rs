use acvm::acir::circuit::Circuit;
use acvm::ProofSystemCompiler;
use std::path::{Path, PathBuf};

use clap::Args;

use crate::{constants::TARGET_DIR, errors::CliError};

use super::fs::{
    keys::save_key_to_dir,
    program::{read_program_from_file, save_acir_hash_to_dir},
};
use super::NargoConfig;

/// Generate proving and verification keys for a circuit.
#[derive(Debug, Clone, Args)]
pub(crate) struct PreprocessCommand {
    /// The name of the program build artifact.
    artifact_name: String,
}

pub(crate) fn run(args: PreprocessCommand, config: NargoConfig) -> Result<(), CliError> {
    let circuit_dir = config.program_dir.join(TARGET_DIR);

    let program = read_program_from_file(circuit_dir.join(&args.artifact_name))?;

    preprocess_with_path(&args.artifact_name, circuit_dir, &program.circuit)?;

    Ok(())
}

pub(crate) fn preprocess_with_path<P: AsRef<Path>>(
    key_name: &str,
    preprocess_dir: P,
    circuit: &Circuit,
) -> Result<(PathBuf, PathBuf), CliError> {
    let backend = nargo_core::backends::ConcreteBackend;
    let (proving_key, verification_key) = backend.preprocess(circuit);

    // Save a checksum of the circuit to compare against during proving and verification.
    // If hash doesn't match then the circuit has been updated and keys are stale.
    save_acir_hash_to_dir(circuit, key_name, &preprocess_dir);

    let pk_path = save_key_to_dir(&proving_key, key_name, &preprocess_dir, true)?;
    let vk_path = save_key_to_dir(&verification_key, key_name, preprocess_dir, false)?;

    Ok((pk_path, vk_path))
}
