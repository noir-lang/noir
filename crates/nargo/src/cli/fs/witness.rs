use std::path::{Path, PathBuf};

use acvm::acir::native_types::Witness;
use noirc_abi::WitnessMap;

use super::{create_named_dir, write_to_file};
use crate::{constants::WITNESS_EXT, errors::CliError};

pub(crate) fn save_witness_to_dir<P: AsRef<Path>>(
    witness: WitnessMap,
    witness_name: &str,
    witness_dir: P,
) -> Result<PathBuf, CliError> {
    let mut witness_path = create_named_dir(witness_dir.as_ref(), "witness");
    witness_path.push(witness_name);
    witness_path.set_extension(WITNESS_EXT);

    let buf = Witness::to_bytes(&witness);

    write_to_file(buf.as_slice(), &witness_path);

    Ok(witness_path)
}
