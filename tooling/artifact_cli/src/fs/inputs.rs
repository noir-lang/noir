use noirc_abi::{
    Abi, InputMap, MAIN_RETURN_NAME,
    input_parser::{Format, InputValue},
};
use std::{collections::BTreeMap, path::Path};

use crate::{errors::CliError, fs::artifact::write_to_file};

/// Returns the circuit's parameters and its return value, if one exists.
///
/// The file is is expected to contain ABI encoded inputs in TOML or JSON format.
pub fn read_inputs_from_file(
    file_path: &Path,
    abi: &Abi,
) -> Result<(InputMap, Option<InputValue>), CliError> {
    use crate::errors::FilesystemError::{InvalidInputFile, MissingInputFile};
    use CliError::FilesystemError;

    let has_params = !abi.parameters.is_empty();
    let has_file = file_path.exists();

    if !has_params && !has_file {
        // Reading a return value from the `Prover.toml` is optional,
        // so if the ABI has no parameters we can skip reading the file if it doesn't exist.
        return Ok((BTreeMap::new(), None));
    }
    if !has_file {
        return Err(FilesystemError(MissingInputFile(file_path.to_path_buf())));
    }

    let format = format_from_file_path(file_path)?;

    let inputs = std::fs::read_to_string(file_path)
        .map_err(|e| FilesystemError(InvalidInputFile(file_path.to_path_buf(), e.to_string())))?;

    let mut inputs = format.parse(&inputs, abi)?;
    let return_value = inputs.remove(MAIN_RETURN_NAME);

    Ok((inputs, return_value))
}

/// Writes input map to a file with a [Format] based on the file extension.
///
/// The inputs are expected to contain the `return` entry, if applicable.
pub fn write_inputs_to_file(
    file_path: &Path,
    abi: &Abi,
    input_map: &InputMap,
) -> Result<(), CliError> {
    let format = format_from_file_path(file_path)?;
    let input_string = format.serialize(input_map, abi)?;
    write_to_file(input_string.as_bytes(), file_path)?;
    Ok(())
}

/// Writes the input map to a file with a given [Format].
pub fn write_inputs_to_file_with_format<P: AsRef<Path>>(
    path: P,
    file_name: &str,
    format: Format,
    abi: &Abi,
    input_map: &InputMap,
) -> Result<(), CliError> {
    if abi.is_empty() {
        return Ok(());
    }
    let file_path = path.as_ref().join(file_name).with_extension(format.ext());
    let input_string = format.serialize(input_map, abi)?;
    write_to_file(input_string.as_bytes(), &file_path)?;
    Ok(())
}

/// Create a [Format] based on the file extension.
fn format_from_file_path(file_path: &Path) -> Result<Format, CliError> {
    use crate::errors::FilesystemError::InvalidInputFile;
    use CliError::FilesystemError;

    let Some(ext) = file_path.extension().and_then(|e| e.to_str()) else {
        return Err(FilesystemError(InvalidInputFile(
            file_path.to_path_buf(),
            "cannot determine input format".to_string(),
        )));
    };

    let Some(format) = Format::from_ext(ext) else {
        return Err(FilesystemError(InvalidInputFile(
            file_path.to_path_buf(),
            format!("unknown input format: {ext}"),
        )));
    };

    Ok(format)
}

#[cfg(test)]
mod tests {
    use super::read_inputs_from_file;

    use crate::errors::CliError;
    use noirc_abi::{Abi, errors::InputParserError};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_input_path() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("noir-artifact-cli-inputs-test-{suffix}.toml"))
    }

    #[test]
    fn rejects_unexpected_keys_in_existing_file_for_empty_abi() {
        let path = temp_input_path();
        fs::write(&path, "extra = \"1\"").unwrap();

        let abi =
            Abi { parameters: Vec::new(), return_type: None, error_types: Default::default() };
        let err = read_inputs_from_file(&path, &abi).unwrap_err();
        fs::remove_file(path).unwrap();

        assert!(matches!(
            err,
            CliError::InputDeserializationError(InputParserError::UnexpectedArguments(keys))
                if keys == vec!["extra".to_string()]
        ));
    }
}
