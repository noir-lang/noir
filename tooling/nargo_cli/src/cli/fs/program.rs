use std::path::{Path, PathBuf};

use nargo::package::CrateName;
use noirc_artifacts::{contract::ContractArtifact, program::ProgramArtifact};

use super::{create_named_dir, write_to_file};

pub(crate) fn save_program_to_file<P: AsRef<Path>>(
    program_artifact: &ProgramArtifact,
    crate_name: &CrateName,
    circuit_dir: P,
) -> PathBuf {
    let circuit_name: String = crate_name.into();
    save_build_artifact_to_file(program_artifact, &circuit_name, circuit_dir)
}

pub(crate) fn save_contract_to_file<P: AsRef<Path>>(
    compiled_contract: &ContractArtifact,
    circuit_name: &str,
    circuit_dir: P,
) -> PathBuf {
    save_build_artifact_to_file(compiled_contract, circuit_name, circuit_dir)
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
