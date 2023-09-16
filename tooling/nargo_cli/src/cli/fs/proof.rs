use std::path::{Path, PathBuf};

use nargo::constants::PROOF_EXT;

use crate::errors::FilesystemError;

use super::{create_named_dir, write_to_file};

pub(crate) fn save_proof_to_dir<P: AsRef<Path>>(
    proof: &[u8],
    proof_name: &str,
    proof_dir: P,
) -> Result<PathBuf, FilesystemError> {
    create_named_dir(proof_dir.as_ref(), "proof");
    let proof_path = proof_dir.as_ref().join(proof_name).with_extension(PROOF_EXT);

    write_to_file(hex::encode(proof).as_bytes(), &proof_path);

    Ok(proof_path)
}
