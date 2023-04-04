use super::{create_named_dir, load_hex_data, write_to_file};
use crate::{
    constants::{ACIR_CHECKSUM, PK_EXT, VK_EXT},
    errors::CliError,
};
use acvm::acir::circuit::Circuit;
use nargo::ops::checksum_acir;
use std::path::{Path, PathBuf};

pub(crate) fn save_key_to_dir<P: AsRef<Path>>(
    key: &[u8],
    key_name: &str,
    key_dir: P,
    is_proving_key: bool,
) -> Result<PathBuf, CliError> {
    create_named_dir(key_dir.as_ref(), key_name);

    let extension = if is_proving_key { PK_EXT } else { VK_EXT };
    let key_path = key_dir.as_ref().join(key_name).with_extension(extension);

    write_to_file(hex::encode(key).as_bytes(), &key_path);

    Ok(key_path)
}

pub(crate) fn fetch_pk_and_vk<P: AsRef<Path>>(
    circuit: &Circuit,
    circuit_build_path: P,
    prove_circuit: bool,
    check_proof: bool,
) -> Result<(Vec<u8>, Vec<u8>), CliError> {
    let acir_hash_path = circuit_build_path.as_ref().with_extension(ACIR_CHECKSUM);

    let expected_acir_checksum = load_hex_data(acir_hash_path.clone())?;
    let new_acir_checksum = checksum_acir(circuit);

    if new_acir_checksum[..] != expected_acir_checksum {
        return Err(CliError::MismatchedAcir(acir_hash_path));
    }

    // This flag exists to avoid an unnecessary read of the proving key during verification
    // as this method is used by both `nargo prove` and `nargo verify`
    let proving_key = if prove_circuit {
        let proving_key_path = circuit_build_path.as_ref().with_extension(PK_EXT);
        load_hex_data(proving_key_path)?
    } else {
        // We can return an empty Vec here as `prove_circuit` should only be false when running `nargo verify`
        vec![]
    };

    let verification_key = if check_proof {
        let verification_key_path = circuit_build_path.as_ref().with_extension(VK_EXT);
        load_hex_data(verification_key_path)?
    } else {
        // We can return an empty Vec here as the verification key is used only is `check_proof` is true
        vec![]
    };

    Ok((proving_key, verification_key))
}

#[cfg(test)]
mod tests {
    use super::fetch_pk_and_vk;
    use crate::cli::fs::{keys::save_key_to_dir, program::save_acir_checksum_to_dir};
    use acvm::acir::circuit::Circuit;
    use nargo::ops::checksum_acir;
    use tempdir::TempDir;

    #[test]
    fn fetching_pk_and_vk_loads_expected_keys() {
        let circuit = Circuit::default();
        let circuit_name = "my_circuit";
        let mut circuit_build_path = TempDir::new("temp_circuit_hash_dir").unwrap().into_path();

        // These values are not meaningful, we just need distinct values.
        let pk: Vec<u8> = vec![0];
        let vk: Vec<u8> = vec![1, 2];
        save_key_to_dir(&pk, circuit_name, &circuit_build_path, true).unwrap();
        save_key_to_dir(&vk, circuit_name, &circuit_build_path, false).unwrap();

        save_acir_checksum_to_dir(checksum_acir(&circuit), circuit_name, &circuit_build_path);
        circuit_build_path.push(circuit_name);

        let loaded_keys = fetch_pk_and_vk(&circuit, circuit_build_path, true, true).unwrap();
        assert_eq!(loaded_keys, (pk, vk));
    }
}
