use noirc_abi::{
    input_parser::{Format, InputValue},
    Abi, InputMap, MAIN_RETURN_NAME,
};
use std::{collections::BTreeMap, path::Path};

use crate::errors::CliError;

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
    let has_return = abi.return_type.is_some();
    let has_file = file_path.exists();

    if !has_params && !has_return {
        return Ok((BTreeMap::new(), None));
    }
    if !has_params && !has_file {
        // Reading a return value from the `Prover.toml` is optional,
        // so if the ABI has no parameters we can skip reading the file if it doesn't exist.
        return Ok((BTreeMap::new(), None));
    }
    if has_params && !has_file {
        return Err(FilesystemError(MissingInputFile(file_path.to_path_buf())));
    }

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

    let inputs = std::fs::read_to_string(file_path)
        .map_err(|e| FilesystemError(InvalidInputFile(file_path.to_path_buf(), e.to_string())))?;

    let mut inputs = format.parse(&inputs, abi)?;
    let return_value = inputs.remove(MAIN_RETURN_NAME);

    Ok((inputs, return_value))
}
