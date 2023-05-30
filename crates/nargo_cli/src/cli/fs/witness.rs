use std::path::{Path, PathBuf};

// use noirc_abi::WitnessMap;
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

    let mut buf = Vec::new();
    for (_, value) in witnesses {
        buf.extend_from_slice(&value.to_be_bytes());
    }

    write_to_file(buf.as_slice(), &witness_path);

    Ok(witness_path)
}
