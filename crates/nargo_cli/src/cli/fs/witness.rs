use std::path::{Path, PathBuf};

use acvm::acir::native_types::WitnessMap;

use super::{create_named_dir, write_to_file};
use crate::{constants::WITNESS_EXT, errors::FilesystemError};

pub(crate) fn save_witness_to_dir<P: AsRef<Path>>(
    witnesses: WitnessMap,
    witness_name: &str,
    witness_dir: P,
) -> Result<PathBuf, FilesystemError> {
    create_named_dir(witness_dir.as_ref(), "witness");
    let witness_path = witness_dir.as_ref().join(witness_name).with_extension(WITNESS_EXT);

    let buf: Vec<u8> = serialize_witness_map(witnesses)?;

    write_to_file(buf.as_slice(), &witness_path);

    Ok(witness_path)
}

#[cfg(not(feature = "flat_witness"))]
fn serialize_witness_map(witnesses: WitnessMap) -> Result<Vec<u8>, FilesystemError> {
    let buf: Vec<u8> = witnesses.try_into()?;
    Ok(buf)
}

#[cfg(feature = "flat_witness")]
fn serialize_witness_map(witnesses: WitnessMap) -> Result<Vec<u8>, FilesystemError> {
    let mut buf: Vec<u8> = Vec::new();
    let mut counter = 1;
    for (index, value) in witnesses {
        while counter < index.witness_index() {
            buf.extend(vec![0; 32]);
            counter += 1;
        }
        buf.extend_from_slice(&value.to_be_bytes());
        counter += 1;
    }
    Ok(buf)
}
