use acir::{
    native_types::{Witness, WitnessMap},
    AcirField, FieldElement,
};
use toml::Table;

use crate::errors::{CliError, FilesystemError};
use std::path::Path;

/// Returns the circuit's parameters parsed from a TOML file at the given location
pub fn read_inputs_from_file(
    work_dir: &Path,
    file_name: &str,
) -> Result<WitnessMap<FieldElement>, CliError> {
    let file_path = work_dir.join(file_name);
    if !file_path.exists() {
        return Err(CliError::FilesystemError(FilesystemError::MissingTomlFile(
            file_name.to_owned(),
            file_path,
        )));
    }

    let input_string = std::fs::read_to_string(file_path)
        .map_err(|_| FilesystemError::InvalidTomlFile(file_name.to_owned()))?;

    let input_map = input_string
        .parse::<Table>()
        .map_err(|_| FilesystemError::InvalidTomlFile(file_name.to_owned()))?;

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
