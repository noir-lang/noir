use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use acir::{
    native_types::{Witness, WitnessStackError},
    FieldElement,
};
use acvm::acir::{
    native_types::{WitnessMap, WitnessStack},
    AcirField,
};

use crate::errors::{CliError, FilesystemError};

/// Returns the circuit's parameters parsed from a TOML file at the given location.
///
/// The expected format is the witness map, not ABI inputs, for example:
/// ```toml
/// "0" = '0x0000000000000000000000000000000000000000000000000000000000100000'
/// "1" = '0x0000000000000000000000000000000000000000000000000000000000000020'
/// "2" = '0x00000000000000000000000000000000000000000000000000000000000328b1'
/// "3" = '0x0000000000000000000000000000000000000000000000000000000000000001'
/// "4" = '0x0000000000000000000000000000000000000000000000000000000000000010'
/// "5" = '0x0000000000000000000000000000000000000000000000000000000000000011'
/// ```
pub fn read_witness_from_file(file_path: &Path) -> Result<WitnessMap<FieldElement>, CliError> {
    if !file_path.exists() {
        return Err(CliError::FilesystemError(FilesystemError::MissingInputFile(
            file_path.to_path_buf(),
        )));
    }

    let input_string = std::fs::read_to_string(&file_path)
        .map_err(|e| FilesystemError::InvalidInputFile(file_path.to_path_buf(), e.to_string()))?;

    let input_map = input_string
        .parse::<toml::Table>()
        .map_err(|e| FilesystemError::InvalidInputFile(file_path.to_path_buf(), e.to_string()))?;

    let mut witnesses: WitnessMap<FieldElement> = WitnessMap::new();

    for (key, value) in input_map.into_iter() {
        let index =
            Witness(key.trim().parse().map_err(|_| CliError::WitnessIndexError(key.clone()))?);
        if !value.is_str() {
            return Err(CliError::WitnessValueError(key.clone()));
        }
        let field = FieldElement::from_hex(value.as_str().unwrap()).unwrap();
        witnesses.insert(index, field);
    }

    Ok(witnesses)
}

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

    let buf: Vec<u8> = witnesses.try_into().map_err(|e: WitnessStackError| {
        FilesystemError::OutputWitnessCreationFailed(witness_path.clone(), format!("{e:?}"))
    })?;

    std::fs::write(&witness_path, buf.as_slice()).map_err(|e| {
        FilesystemError::OutputWitnessCreationFailed(witness_path.clone(), e.to_string())
    })?;

    Ok(witness_path)
}
