use std::path::{Path, PathBuf};

use noirc_abi::WitnessMap;

use super::{create_named_dir, write_to_file};
use crate::{constants::WITNESS_EXT, errors::FilesystemError};


pub(crate) fn save_witness_to_dir<P: AsRef<Path>>(
    witnesses: WitnessMap,
    witness_name: &str,
    witness_dir: P,
) -> Result<PathBuf, FilesystemError> {
    create_named_dir(witness_dir.as_ref(), "witness");
    let witness_path = witness_dir.as_ref().join(witness_name).with_extension(WITNESS_EXT);

    // let buf = Witness::to_bytes(&witness);
    let mut buf = Vec::new();
    for witness in witnesses.values() {
        buf.extend_from_slice(&witness.to_be_bytes());
    }

    write_to_file(buf.as_slice(), &witness_path);

    Ok(witness_path)
}
