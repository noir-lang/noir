use std::path::{Path, PathBuf};

use acvm::{acir::native_types::WitnessStack, FieldElement};
use nargo::constants::WITNESS_EXT;

use super::{create_named_dir, write_to_file};
use crate::errors::FilesystemError;

pub(crate) fn save_witness_to_dir<P: AsRef<Path>>(
    witness_stack: WitnessStack<FieldElement>,
    witness_name: &str,
    witness_dir: P,
) -> Result<PathBuf, FilesystemError> {
    create_named_dir(witness_dir.as_ref(), "witness");
    let witness_path = witness_dir.as_ref().join(witness_name).with_extension(WITNESS_EXT);

    let buf: Vec<u8> = witness_stack.try_into()?;

    write_to_file(buf.as_slice(), &witness_path);

    Ok(witness_path)
}
