use std::path::{Path, PathBuf};

use acvm::acir::circuit::Circuit;
use nargo::artifacts::{
    contract::PreprocessedContract, debug::DebugArtifact, program::PreprocessedProgram,
};
use noirc_frontend::graph::CrateName;

use crate::errors::FilesystemError;

use super::{create_named_dir, write_to_file};

pub(crate) fn save_program_to_file<P: AsRef<Path>>(
    compiled_program: &PreprocessedProgram,
    crate_name: &CrateName,
    circuit_dir: P,
) -> PathBuf {
    let circuit_name: String = crate_name.into();
    save_build_artifact_to_file(compiled_program, &circuit_name, circuit_dir)
}

/// Writes the bytecode as acir.gz
pub(crate) fn only_acir<P: AsRef<Path>>(
    compiled_program: &PreprocessedProgram,
    circuit_dir: P,
) -> PathBuf {
    create_named_dir(circuit_dir.as_ref(), "target");
    let circuit_path = circuit_dir.as_ref().join("acir").with_extension("gz");
    let bytes = Circuit::serialize_circuit(&compiled_program.bytecode);
    write_to_file(&bytes, &circuit_path);

    circuit_path
}

pub(crate) fn save_contract_to_file<P: AsRef<Path>>(
    compiled_contract: &PreprocessedContract,
    circuit_name: &str,
    circuit_dir: P,
) -> PathBuf {
    save_build_artifact_to_file(compiled_contract, circuit_name, circuit_dir)
}

pub(crate) fn save_debug_artifact_to_file<P: AsRef<Path>>(
    debug_artifact: &DebugArtifact,
    circuit_name: &str,
    circuit_dir: P,
) -> PathBuf {
    let artifact_name = format!("debug_{circuit_name}");
    save_build_artifact_to_file(debug_artifact, &artifact_name, circuit_dir)
}

fn save_build_artifact_to_file<P: AsRef<Path>, T: ?Sized + serde::Serialize>(
    build_artifact: &T,
    artifact_name: &str,
    circuit_dir: P,
) -> PathBuf {
    create_named_dir(circuit_dir.as_ref(), "target");
    let circuit_path = circuit_dir.as_ref().join(artifact_name).with_extension("json");

    write_to_file(&serde_json::to_vec(build_artifact).unwrap(), &circuit_path);

    circuit_path
}

pub(crate) fn read_program_from_file<P: AsRef<Path>>(
    circuit_path: P,
) -> Result<PreprocessedProgram, FilesystemError> {
    let file_path = circuit_path.as_ref().with_extension("json");

    let input_string =
        std::fs::read(&file_path).map_err(|_| FilesystemError::PathNotValid(file_path))?;
    let program = serde_json::from_slice(&input_string)
        .map_err(|err| FilesystemError::ProgramSerializationError(err.to_string()))?;

    Ok(program)
}

pub(crate) fn read_debug_artifact_from_file<P: AsRef<Path>>(
    debug_artifact_path: P,
) -> Result<DebugArtifact, FilesystemError> {
    let input_string = std::fs::read(&debug_artifact_path)
        .map_err(|_| FilesystemError::PathNotValid(debug_artifact_path.as_ref().into()))?;
    let program = serde_json::from_slice(&input_string)
        .map_err(|err| FilesystemError::ProgramSerializationError(err.to_string()))?;

    Ok(program)
}
