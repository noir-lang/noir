use iter_extended::try_btree_map;
use noirc_abi::{
    input_parser::{Format, InputValue},
    Abi, InputMap, MAIN_RETURN_NAME,
};
use std::{collections::BTreeMap, path::Path};

use crate::errors::CliError;

use super::write_to_file;

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
) -> Result<(InputMap, Option<InputValue>), CliError> {
    if abi.is_empty() {
        return Ok((BTreeMap::new(), None));
    }

    let file_path = {
        let mut dir_path = path.as_ref().to_path_buf();
        dir_path.push(file_name);
        dir_path.set_extension(format.ext());
        dir_path
    };
    if !file_path.exists() {
        return Err(CliError::MissingTomlFile(file_name.to_owned(), file_path));
    }

    let input_string = std::fs::read_to_string(file_path).unwrap();
    let mut input_map = format.parse(&input_string, abi)?;
    let return_value = input_map.remove(MAIN_RETURN_NAME);

    Ok((input_map, return_value))
}

/// Returns the circuit's parameters and the return value, when the inputs
/// come from the command line.
///
/// ```ignore
/// let (input_map, return_value): (InputMap, Option<InputValue>) =
///  read_inputs_from_cli(inputs, &abi)?;
/// ```
pub(crate) fn read_inputs_from_cli(
    inputs: Vec<(String, String)>,
    abi: &Abi,
) -> Result<(InputMap, Option<InputValue>), CliError> {
    if abi.is_empty() {
        return Ok((BTreeMap::new(), None));
    }

    let abi_map = &abi.to_btree_map();
    let mut input_map = try_btree_map(inputs, |(key, value)| {
        InputValue::try_from_cli_args(value, &abi_map[&key]).map(|input_value| (key, input_value))
    })?;
    let return_value = input_map.remove(MAIN_RETURN_NAME);

    Ok((input_map, return_value))
}

pub(crate) fn write_inputs_to_file<P: AsRef<Path>>(
    input_map: &InputMap,
    return_value: &Option<InputValue>,
    path: P,
    file_name: &str,
    format: Format,
) -> Result<(), CliError> {
    let file_path = {
        let mut dir_path = path.as_ref().to_path_buf();
        dir_path.push(file_name);
        dir_path.set_extension(format.ext());
        dir_path
    };

    // We must insert the return value into the `InputMap` in order for it to be written to file.
    let serialized_output = match return_value {
        // Parameters and return values are kept separate except for when they're being written to file.
        // As a result, we don't want to modify the original map and must clone it before insertion.
        Some(return_value) => {
            let mut input_map = input_map.clone();
            input_map.insert(MAIN_RETURN_NAME.to_owned(), return_value.clone());
            format.serialize(&input_map)?
        }
        // If no return value exists, then we can serialize the original map directly.
        None => format.serialize(input_map)?,
    };

    write_to_file(serialized_output.as_bytes(), &file_path);

    Ok(())
}
