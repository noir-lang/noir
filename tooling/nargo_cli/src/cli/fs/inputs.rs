use noirc_abi::{
    input_parser::{Format, InputValue},
    Abi, InputMap, MAIN_RETURN_NAME,
};
use std::{collections::BTreeMap, path::Path};

use crate::errors::FilesystemError;

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
) -> Result<(InputMap, Vec<InputValue>), FilesystemError> {
    if abi.is_empty() {
        return Ok((BTreeMap::new(), vec![]));
    }

    let file_path = path.as_ref().join(file_name).with_extension(format.ext());
    if !file_path.exists() {
        return Err(FilesystemError::MissingTomlFile(file_name.to_owned(), file_path));
    }

    let input_string = std::fs::read_to_string(file_path).unwrap();
    let mut input_map = format.parse(&input_string, abi)?;
    let return_value = match input_map.remove(MAIN_RETURN_NAME) {
        None => vec![],
        Some(InputValue::Vec(return_values)) => return_values,
        _ => unreachable!("Badly formed return values"),
    };

    Ok((input_map, return_value))
}

pub(crate) fn write_inputs_to_file<P: AsRef<Path>>(
    input_map: &InputMap,
    return_value: &[InputValue],
    abi: &Abi,
    path: P,
    file_name: &str,
    format: Format,
) -> Result<(), FilesystemError> {
    let file_path = path.as_ref().join(file_name).with_extension(format.ext());

    // We must insert the return value into the `InputMap` in order for it to be written to file.
    let serialized_output = if return_value.len() > 0 {
        // Parameters and return values are kept separate except for when they're being written to file.
        // As a result, we don't want to modify the original map and must clone it before insertion.

        let mut input_map = input_map.clone();
        input_map.insert(MAIN_RETURN_NAME.to_owned(), InputValue::Vec(return_value.to_vec()));
        format.serialize(&input_map, abi)?
    } else {
        // If no return value exists, then we can serialize the original map directly.
        format.serialize(input_map, abi)?
    };

    write_to_file(serialized_output.as_bytes(), &file_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, vec};

    use acvm::FieldElement;
    use nargo::constants::VERIFIER_INPUT_FILE;
    use noirc_abi::{
        input_parser::{Format, InputValue},
        Abi, AbiParameter, AbiReturnType, AbiType, AbiVisibility,
    };
    use tempfile::TempDir;

    use super::{read_inputs_from_file, write_inputs_to_file};

    #[test]
    fn write_and_read_recovers_inputs_and_return_value() {
        let input_dir = TempDir::new().unwrap().into_path();

        // We purposefully test a simple ABI here as we're focussing on `fs`.
        // Tests for serializing complex types should exist in `noirc_abi`.
        let abi = Abi {
            parameters: vec![
                AbiParameter {
                    name: "foo".into(),
                    typ: AbiType::Field,
                    visibility: AbiVisibility::Public,
                },
                AbiParameter {
                    name: "bar".into(),
                    typ: AbiType::String { length: 11 },
                    visibility: AbiVisibility::Private,
                },
            ],
            return_type: Some(AbiReturnType {
                abi_type: AbiType::Field,
                visibility: AbiVisibility::Public,
            }),

            // Input serialization is only dependent on types, not position in witness map.
            // Neither of these should be relevant so we leave them empty.
            param_witnesses: BTreeMap::new(),
            return_witnesses: Vec::new(),
        };
        let input_map = BTreeMap::from([
            ("foo".to_owned(), InputValue::Field(42u128.into())),
            ("bar".to_owned(), InputValue::String("hello world".to_owned())),
        ]);
        let return_value = Some(InputValue::Field(FieldElement::zero()));

        write_inputs_to_file(
            &input_map,
            &return_value,
            &abi,
            &input_dir,
            VERIFIER_INPUT_FILE,
            Format::Toml,
        )
        .unwrap();

        let (loaded_inputs, loaded_return_value) =
            read_inputs_from_file(input_dir, VERIFIER_INPUT_FILE, Format::Toml, &abi).unwrap();

        assert_eq!(loaded_inputs, input_map);
        assert_eq!(loaded_return_value, return_value);
    }
}
