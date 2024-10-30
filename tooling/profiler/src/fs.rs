use std::{collections::BTreeMap, path::Path};

use color_eyre::eyre;
use noirc_abi::{
    input_parser::{Format, InputValue},
    Abi, InputMap, MAIN_RETURN_NAME,
};
use noirc_artifacts::program::ProgramArtifact;

pub(crate) fn read_program_from_file<P: AsRef<Path>>(
    circuit_path: P,
) -> eyre::Result<ProgramArtifact> {
    let file_path = circuit_path.as_ref().with_extension("json");

    let input_string = std::fs::read(file_path)?;
    let program = serde_json::from_slice(&input_string)?;

    Ok(program)
}

/// Returns the circuit's parameters and its return value, if one exists.
/// # Examples
///
/// ```ignore
/// let (input_map, return_value): (InputMap, Option<InputValue>) =
///   read_inputs_from_file(path, "Verifier", Format::Toml, &abi)?;
/// ```
pub(crate) fn read_inputs_from_file(
    file_path: &Path,
    format: Format,
    abi: &Abi,
) -> eyre::Result<(InputMap, Option<InputValue>)> {
    if abi.is_empty() {
        return Ok((BTreeMap::new(), None));
    }

    let input_string = std::fs::read_to_string(file_path)?;
    let mut input_map = format.parse(&input_string, abi)?;
    let return_value = input_map.remove(MAIN_RETURN_NAME);

    Ok((input_map, return_value))
}
