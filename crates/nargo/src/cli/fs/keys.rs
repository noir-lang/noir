use std::path::{Path, PathBuf};

use acvm::{acir::circuit::Circuit, hash_constraint_system};

use crate::{
    constants::{PK_EXT, VK_EXT},
    errors::CliError,
};

use super::{create_named_dir, load_hex_data, write_to_file};

pub(crate) fn save_key_to_dir<P: AsRef<Path>>(
    key: Vec<u8>,
    key_name: &str,
    key_dir: P,
    is_proving_key: bool,
) -> Result<PathBuf, CliError> {
    let mut key_path = create_named_dir(key_dir.as_ref(), key_name);
    key_path.push(key_name);
    let extension = if is_proving_key { PK_EXT } else { VK_EXT };
    key_path.set_extension(extension);

    write_to_file(hex::encode(key).as_bytes(), &key_path);

    Ok(key_path)
}

pub(crate) fn fetch_pk_and_vk<P: AsRef<Path>>(
    circuit: &Circuit,
    circuit_build_path: P,
    prove_circuit: bool,
    check_proof: bool,
) -> Result<(Vec<u8>, Vec<u8>), CliError> {
    let mut acir_hash_path = PathBuf::from(circuit_build_path.as_ref());
    acir_hash_path.set_extension("json.checksum");

    let expected_acir_hash = load_hex_data(acir_hash_path.clone())?;

    let new_acir_hash = hash_constraint_system(circuit);

    if new_acir_hash[..] != expected_acir_hash {
        return Err(CliError::MismatchedAcir(acir_hash_path));
    }

    // This flag exists to avoid an unnecessary read of the proving key during verification
    // as this method is used by both `nargo prove` and `nargo verify`
    let proving_key = if prove_circuit {
        let mut proving_key_path = PathBuf::new();
        proving_key_path.push(circuit_build_path.as_ref());
        proving_key_path.set_extension(PK_EXT);
        load_hex_data(proving_key_path)?
    } else {
        // We can return an empty Vec here as `prove_circuit` should only be false when running `nargo verify`
        vec![]
    };

    let verification_key = if check_proof {
        let mut verification_key_path = PathBuf::new();
        verification_key_path.push(circuit_build_path);
        verification_key_path.set_extension(VK_EXT);
        load_hex_data(verification_key_path)?
    } else {
        // We can return an empty Vec here as the verification key is used only is `check_proof` is true
        vec![]
    };

    Ok((proving_key, verification_key))
}
