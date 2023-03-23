use std::path::{Path, PathBuf};

use acvm::{acir::circuit::Circuit, hash_constraint_system};
use noirc_driver::{CompiledContract, CompiledProgram};

use crate::{constants::ACIR_CHECKSUM, errors::CliError};

use super::{create_named_dir, write_to_file};

pub(crate) fn save_program_to_file<P: AsRef<Path>>(
    compiled_program: &CompiledProgram,
    circuit_name: &str,
    circuit_dir: P,
) -> PathBuf {
    save_build_artifact_to_file(compiled_program, circuit_name, circuit_dir)
}
pub(crate) fn save_contract_to_file<P: AsRef<Path>>(
    compiled_contract: &CompiledContract,
    circuit_name: &str,
    circuit_dir: P,
) -> PathBuf {
    save_build_artifact_to_file(compiled_contract, circuit_name, circuit_dir)
}
fn save_build_artifact_to_file<P: AsRef<Path>, T: ?Sized + serde::Serialize>(
    strukt: &T,
    circuit_name: &str,
    circuit_dir: P,
) -> PathBuf {
    create_named_dir(circuit_dir.as_ref(), "target");
    let circuit_path = circuit_dir.as_ref().join(circuit_name).with_extension("json");

    write_to_file(&serde_json::to_vec(strukt).unwrap(), &circuit_path);

    circuit_path
}

pub(crate) fn save_acir_hash_to_dir<P: AsRef<Path>>(
    circuit: &Circuit,
    hash_name: &str,
    hash_dir: P,
) -> PathBuf {
    let acir_hash = hash_constraint_system(circuit);
    let hash_path = hash_dir.as_ref().join(hash_name).with_extension(ACIR_CHECKSUM);
    write_to_file(hex::encode(acir_hash).as_bytes(), &hash_path);

    hash_path
}

pub(crate) fn read_program_from_file<P: AsRef<Path>>(
    circuit_path: P,
) -> Result<CompiledProgram, CliError> {
    let file_path = circuit_path.as_ref().with_extension("json");

    let input_string = std::fs::read(&file_path).map_err(|_| CliError::PathNotValid(file_path))?;

    let program = serde_json::from_slice(&input_string).expect("could not deserialize program");

    Ok(program)
}
