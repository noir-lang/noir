use std::path::Path;

use color_eyre::eyre;
use noirc_artifacts::program::ProgramArtifact;

pub(crate) fn read_program_from_file<P: AsRef<Path>>(
    circuit_path: P,
) -> eyre::Result<ProgramArtifact> {
    let file_path = circuit_path.as_ref().with_extension("json");

    let input_string = std::fs::read(file_path)?;
    let program = serde_json::from_slice(&input_string)?;

    Ok(program)
}
