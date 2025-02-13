use std::path::Path;

use crate::{
    errors::{CliError, FilesystemError},
    Artifact,
};
use noirc_artifacts::contract::ContractArtifact;
use noirc_artifacts::program::ProgramArtifact;

impl Artifact {
    /// Try to parse an artifact as a binary program or a contract
    pub fn read_from_file(path: &Path) -> Result<Self, CliError> {
        let json = std::fs::read(path).map_err(FilesystemError::from)?;

        let as_program = || serde_json::from_slice::<ProgramArtifact>(&json).map(Artifact::Program);
        let as_contract =
            || serde_json::from_slice::<ContractArtifact>(&json).map(Artifact::Contract);

        as_program()
            .or_else(|e| as_contract().map_err(|_| e))
            .map_err(CliError::ArtifactDeserializationError)
    }
}

/// Returns the circuit's bytecode read from the file at the given location
pub fn read_bytecode_from_file(
    work_dir: &Path,
    file_name: &str,
) -> Result<Vec<u8>, FilesystemError> {
    let file_path = work_dir.join(file_name);
    if !file_path.exists() {
        return Err(FilesystemError::MissingBytecodeFile(file_name.to_owned(), file_path));
    }
    let bytecode: Vec<u8> = std::fs::read(file_path)
        .map_err(|_| FilesystemError::InvalidBytecodeFile(file_name.to_owned()))?;
    Ok(bytecode)
}
