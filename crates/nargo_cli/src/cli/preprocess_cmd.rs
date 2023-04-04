use acvm::acir::circuit::Circuit;
use acvm::ProofSystemCompiler;
use std::path::{Path, PathBuf};

use clap::Args;

use crate::{constants::TARGET_DIR, errors::CliError};

use super::fs::{
    keys::save_key_to_dir,
    program::{hash_acir, read_program_from_file, save_acir_hash_to_dir},
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
    let preprocess_data = PreprocessedData::from(&program.circuit);
    save_preprocess_data(&preprocess_data, &args.artifact_name, circuit_dir)?;

    Ok(())
}
/// The result of preprocessing the ACIR bytecode.
/// The proving, verification key and circuit are backend specific.
///
/// The circuit is backend specific because at the end of compilation
/// an optimization pass is applied which will transform the bytecode into
/// a format that the backend will accept; removing unsupported gates
/// is one example of this.
pub(crate) struct PreprocessedData {
    pub(crate) proving_key: Vec<u8>,
    pub(crate) verification_key: Vec<u8>,
    pub(crate) program_hash: [u8; 32],
}

impl From<&Circuit> for PreprocessedData {
    fn from(circuit: &Circuit) -> Self {
        let backend = crate::backends::ConcreteBackend;
        let (proving_key, verification_key) = backend.preprocess(circuit);
        let program_hash = hash_acir(circuit);

        PreprocessedData { proving_key, verification_key, program_hash }
    }
}

pub(crate) fn save_preprocess_data<P: AsRef<Path>>(
    data: &PreprocessedData,
    key_name: &str,
    preprocess_dir: P,
) -> Result<(PathBuf, PathBuf), CliError> {
    // Save a checksum of the circuit to compare against during proving and verification.
    // If hash doesn't match then the circuit has been updated and keys are stale.
    save_acir_hash_to_dir(data.program_hash, key_name, &preprocess_dir);

    let pk_path = save_key_to_dir(&data.proving_key, key_name, &preprocess_dir, true)?;
    let vk_path = save_key_to_dir(&data.verification_key, key_name, preprocess_dir, false)?;

    Ok((pk_path, vk_path))
}
