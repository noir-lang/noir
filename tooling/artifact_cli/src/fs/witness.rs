use std::path::{Path, PathBuf};

use acir::FieldElement;
use acvm::acir::native_types::WitnessStack;

use crate::{
    errors::{CliError, FilesystemError},
    fs::artifact::write_to_file,
};

/// Write `witness.gz` to the output directory.
pub fn save_witness_to_dir(
    witnesses: &WitnessStack<FieldElement>,
    witness_name: &str,
    witness_dir: &Path,
) -> Result<PathBuf, CliError> {
    std::fs::create_dir_all(witness_dir)?;

    let witness_path = witness_dir.join(witness_name).with_extension("gz");

    let buf: Vec<u8> = witnesses.serialize().map_err(|e| {
        FilesystemError::OutputWitnessCreationFailed(witness_path.clone(), format!("{e:?}"))
    })?;

    write_to_file(buf.as_slice(), &witness_path).map_err(|e| {
        FilesystemError::OutputWitnessCreationFailed(witness_path.clone(), e.to_string())
    })?;

    Ok(witness_path)
}

pub fn load_witness_from_file(witness_path: &Path) -> Result<WitnessStack<FieldElement>, CliError> {
    let witness_data = std::fs::read(witness_path)?;

    Ok(WitnessStack::deserialize(&witness_data).map_err(|e| {
        FilesystemError::InvalidInputFile(witness_path.to_path_buf(), e.to_string())
    })?)
}
