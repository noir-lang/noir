use std::path::{Path, PathBuf};

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
    let artifact_name = format!("debug-{}", circuit_name);
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

    let program = serde_json::from_slice(&input_string).expect("could not deserialize program");

    Ok(program)
}
