use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use acvm::FieldElement;
use acvm::acir::circuit::Program;
use noirc_abi::Abi;

use crate::{
    Artifact,
    errors::{CliError, FilesystemError},
};
use noirc_artifacts::contract::ContractArtifact;
use noirc_artifacts::program::{CompiledProgram, ProgramArtifact};
use noirc_driver::CrateName;
use serde::de::Error;

impl Artifact {
    /// Try to parse an artifact as a binary program or a contract
    pub fn read_from_file(path: &Path) -> Result<Self, CliError> {
        let file = path.with_extension("json");
        let json = std::fs::read(&file)
            .map_err(|err| FilesystemError::FailedToReadFile(file.clone(), err))?;

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
        return Err(FilesystemError::MissingBytecodeFile(file_path));
    }
    let bytecode: Vec<u8> = std::fs::read(&file_path)
        .map_err(|e| FilesystemError::InvalidBytecodeFile(file_path, e.to_string()))?;
    Ok(bytecode)
}

/// Read a `ProgramArtifact`. Returns error if it turns out to be a `ContractArtifact`.
pub fn read_program_from_file(path: &Path) -> Result<ProgramArtifact, CliError> {
    match Artifact::read_from_file(path)? {
        Artifact::Program(program) => Ok(program),
        Artifact::Contract(contract) => {
            let msg = format!(
                "expected a program artifact but found a contract in {}: {}",
                path.display(),
                contract.name
            );
            Err(CliError::ArtifactDeserializationError(serde_json::Error::custom(msg)))
        }
    }
}

pub fn save_program_to_file(
    program_artifact: &ProgramArtifact,
    crate_name: &CrateName,
    output_dir: &Path,
) -> Result<PathBuf, CliError> {
    let circuit_name: String = crate_name.into();
    save_build_artifact_to_file(program_artifact, &circuit_name, output_dir)
        .map_err(|err| CliError::FailedToSaveProgram(circuit_name, Box::new(err)))
}

pub fn save_contract_to_file(
    compiled_contract: &ContractArtifact,
    circuit_name: &str,
    output_dir: &Path,
) -> Result<PathBuf, CliError> {
    save_build_artifact_to_file(compiled_contract, circuit_name, output_dir)
        .map_err(|err| CliError::FailedToSaveContract(circuit_name.to_string(), Box::new(err)))
}

fn save_build_artifact_to_file<T: ?Sized + serde::Serialize>(
    build_artifact: &T,
    artifact_name: &str,
    output_dir: &Path,
) -> Result<PathBuf, CliError> {
    let artifact_path = output_dir.join(artifact_name).with_extension("json");
    let bytes = serde_json::to_vec(build_artifact)?;
    write_to_file(&bytes, &artifact_path)?;
    Ok(artifact_path)
}

/// Load a [`CompiledProgram`] from separate bytecode and ABI files.
///
/// The bytecode file contains the raw gzip-compressed program (the same binary format
/// produced by [`Program::serialize_program`]). The ABI file is plain JSON.
///
/// Debug symbols and source maps are not available in this mode, so error
/// diagnostics will not include source-level stack traces.
pub fn load_program_from_parts(
    bytecode_path: &Path,
    abi_path: &Path,
) -> Result<CompiledProgram, CliError> {
    let bytecode_bytes = std::fs::read(bytecode_path).map_err(|e| {
        FilesystemError::InvalidBytecodeFile(bytecode_path.to_path_buf(), e.to_string())
    })?;
    let program = Program::<FieldElement>::deserialize_program(&bytecode_bytes)?;

    let abi_bytes = std::fs::read(abi_path)
        .map_err(|e| FilesystemError::InvalidInputFile(abi_path.to_path_buf(), e.to_string()))?;
    let abi: Abi = serde_json::from_slice(&abi_bytes)?;

    Ok(CompiledProgram {
        noir_version: String::new(),
        hash: 0,
        program,
        abi,
        debug: vec![],
        file_map: BTreeMap::new(),
        warnings: vec![],
    })
}

/// Create the parent directory if needed and write the bytes to a file.
pub fn write_to_file(bytes: &[u8], path: &Path) -> Result<(), FilesystemError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|err| FilesystemError::FailedToCreateDirectory(dir.to_path_buf(), err))?;
    }
    std::fs::write(path, bytes)
        .map_err(|err| FilesystemError::FailedToWriteFile(path.to_path_buf(), err))?;
    Ok(())
}
