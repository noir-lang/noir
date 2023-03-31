use std::path::{Path, PathBuf};

use crate::{constants::PROOF_EXT, errors::CliError};

use super::{create_named_dir, write_to_file};

pub(crate) fn save_proof_to_dir<P: AsRef<Path>>(
    proof: &[u8],
    proof_name: &str,
    proof_dir: P,
) -> Result<PathBuf, CliError> {
    create_named_dir(proof_dir.as_ref(), "proof");
    let proof_path = proof_dir.as_ref().join(proof_name).with_extension(PROOF_EXT);

    write_to_file(hex::encode(proof).as_bytes(), &proof_path);

    Ok(proof_path)
}
