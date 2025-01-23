use noirc_abi::{
    input_parser::{Format, InputValue},
    Abi, InputMap, MAIN_RETURN_NAME,
};
use std::{collections::BTreeMap, path::Path};
use strum::IntoEnumIterator;

use crate::errors::FilesystemError;

/// Returns the circuit's parameters and its return value, if one exists.
/// # Examples
///
/// ```ignore
/// let (input_map, return_value): (InputMap, Option<InputValue>) =
///   read_inputs_from_file(path, "Verifier", Format::Toml, &abi)?;
/// ```
pub(crate) fn read_inputs_from_file<P: AsRef<Path>>(
    path: P,
    file_name: &str,
    format: Format,
    abi: &Abi,
) -> Result<(InputMap, Option<InputValue>), FilesystemError> {
    if abi.is_empty() {
        return Ok((BTreeMap::new(), None));
    }

    let file_path = path.as_ref().join(file_name).with_extension(format.ext());
    if !file_path.exists() {
        if abi.parameters.is_empty() {
            // Reading a return value from the `Prover.toml` is optional,
            // so if the ABI has no parameters we can skip reading the file if it doesn't exist.
            return Ok((BTreeMap::new(), None));
        } else {
            return Err(FilesystemError::MissingTomlFile(file_name.to_owned(), file_path));
        }
    }

    let input_string = std::fs::read_to_string(file_path).unwrap();
    let mut input_map = format.parse(&input_string, abi)?;
    let return_value = input_map.remove(MAIN_RETURN_NAME);

    Ok((input_map, return_value))
}

/// Try to look for any format with the given file name:
/// 1. TOML
/// 2. JSON
pub(crate) fn read_inputs_from_file_any_format(
    path: &Path,
    file_name: &str,
    abi: &Abi,
) -> Result<(InputMap, Option<InputValue>), FilesystemError> {
    let mut first_missing = None;
    for format in Format::iter() {
        match read_inputs_from_file(path, file_name, format, abi) {
            Err(e @ FilesystemError::MissingTomlFile(..)) => {
                if first_missing.is_none() {
                    first_missing = Some(e);
                }
            }
            other => {
                return other;
            }
        }
    }
    Err(first_missing.expect("must have encountered an error"))
}
