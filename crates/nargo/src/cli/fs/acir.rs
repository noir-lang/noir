use std::path::{Path, PathBuf};

use acvm::{acir::circuit::Circuit, hash_constraint_system};

use crate::constants::ACIR_EXT;

use super::{create_named_dir, write_to_file};

pub(crate) fn save_acir_to_dir<P: AsRef<Path>>(
    circuit: &Circuit,
    circuit_name: &str,
    circuit_dir: P,
) -> PathBuf {
    let mut circuit_path = create_named_dir(circuit_dir.as_ref(), "target");
    circuit_path.push(circuit_name);

    let mut serialized = Vec::new();
    circuit.write(&mut serialized).expect("could not serialize circuit");

    circuit_path.set_extension(ACIR_EXT);
    write_to_file(serialized.as_slice(), &circuit_path);

    // Save a checksum of the circuit to compare against during proving and verification
    let acir_hash = hash_constraint_system(circuit);
    circuit_path.set_extension(ACIR_EXT.to_owned() + ".sha256");
    write_to_file(hex::encode(acir_hash).as_bytes(), &circuit_path);

    circuit_path
}
