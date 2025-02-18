use std::path::{Path, PathBuf};

use acir::{native_types::WitnessStackError, FieldElement};
use acvm::acir::native_types::WitnessStack;

use crate::errors::FilesystemError;

/// Write `witness.gz` to the output directory.
pub fn save_witness_to_dir(
    witnesses: WitnessStack<FieldElement>,
    witness_name: &str,
    witness_dir: &Path,
) -> Result<PathBuf, FilesystemError> {
    std::fs::create_dir_all(witness_dir)?;

    let witness_path = witness_dir.join(witness_name).with_extension("gz");

    let buf: Vec<u8> = witnesses.try_into().map_err(|e: WitnessStackError| {
        FilesystemError::OutputWitnessCreationFailed(witness_path.clone(), format!("{e:?}"))
    })?;

    std::fs::write(&witness_path, buf.as_slice()).map_err(|e| {
        FilesystemError::OutputWitnessCreationFailed(witness_path.clone(), e.to_string())
    })?;

    Ok(witness_path)
}
