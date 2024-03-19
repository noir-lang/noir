use std::{
    collections::BTreeMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use acvm::acir::native_types::WitnessMap;

use crate::errors::{CliError, FilesystemError};

/// Saves the provided output witnesses to a toml file created at the given location
pub(crate) fn save_witness_to_dir<P: AsRef<Path>>(
    output_witness: &String,
    witness_dir: P,
    file_name: &String,
) -> Result<PathBuf, FilesystemError> {
    let witness_path = witness_dir.as_ref().join(file_name);

    let mut file = File::create(&witness_path)
        .map_err(|_| FilesystemError::OutputWitnessCreationFailed(file_name.clone()))?;
    write!(file, "{}", output_witness)
        .map_err(|_| FilesystemError::OutputWitnessWriteFailed(file_name.clone()))?;

    Ok(witness_path)
}

/// Creates a toml representation of the provided witness map
pub(crate) fn create_output_witness_string(witnesses: &WitnessMap) -> Result<String, CliError> {
    let mut witness_map: BTreeMap<String, String> = BTreeMap::new();
    for (key, value) in witnesses.clone().into_iter() {
        witness_map.insert(key.0.to_string(), format!("0x{}", value.to_hex()));
    }

    toml::to_string(&witness_map).map_err(|_| CliError::OutputWitnessSerializationFailed())
}
