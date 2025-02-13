use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use acir::FieldElement;
use acvm::acir::{
    native_types::{WitnessMap, WitnessStack},
    AcirField,
};

use crate::errors::{CliError, FilesystemError};

/// Creates a TOML representation of the provided witness map
pub fn create_output_witness_string(
    witnesses: &WitnessMap<FieldElement>,
) -> Result<String, CliError> {
    let mut witness_map: BTreeMap<String, String> = BTreeMap::new();
    for (key, value) in witnesses.clone().into_iter() {
        witness_map.insert(key.0.to_string(), format!("0x{}", value.to_hex()));
    }
    toml::to_string(&witness_map).map_err(CliError::OutputWitnessSerializationFailed)
}

/// Write `witness.gz` to the output directory.
pub fn save_witness_to_dir(
    witnesses: WitnessStack<FieldElement>,
    witness_name: &str,
    witness_dir: &Path,
) -> Result<PathBuf, FilesystemError> {
    std::fs::create_dir_all(witness_dir)?;

    let witness_path = witness_dir.join(witness_name).with_extension("gz");

    let buf: Vec<u8> = witnesses
        .try_into()
        .map_err(|_op| FilesystemError::OutputWitnessCreationFailed(witness_name.to_string()))?;

    std::fs::write(&witness_path, buf.as_slice())?;

    Ok(witness_path)
}
