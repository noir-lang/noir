use std::path::{Path, PathBuf};

use acvm::hash_constraint_system;
use noirc_driver::CompiledProgram;

use super::{create_named_dir, write_to_file};

pub(crate) fn save_program_to_file<P: AsRef<Path>>(
    compiled_program: &CompiledProgram,
    circuit_name: &str,
    circuit_dir: P,
) -> PathBuf {
    let mut circuit_path = create_named_dir(circuit_dir.as_ref(), "target");
    circuit_path.push(circuit_name);
    circuit_path.set_extension("json");

    write_to_file(&serde_json::to_vec(compiled_program).unwrap(), &circuit_path);

    // Save a checksum of the circuit to compare against during proving and verification
    let acir_hash = hash_constraint_system(&compiled_program.circuit);
    circuit_path.set_extension("json.sha256");
    write_to_file(hex::encode(acir_hash).as_bytes(), &circuit_path);

    circuit_path
}

pub(crate) fn read_program_from_file<P: AsRef<Path>>(circuit_path: P) -> CompiledProgram {
    let file_path = circuit_path.as_ref().with_extension("json");

    let input_string = std::fs::read(file_path).unwrap();

    serde_json::from_slice(&input_string).unwrap()
}
