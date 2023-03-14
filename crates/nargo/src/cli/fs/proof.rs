use std::path::{Path, PathBuf};

use crate::{constants::PROOF_EXT, errors::CliError};

use super::{create_named_dir, write_to_file};

pub(crate) fn save_proof_to_dir<P: AsRef<Path>>(
    proof: &[u8],
    proof_name: &str,
    proof_dir: P,
) -> Result<PathBuf, CliError> {
    let mut proof_path = create_named_dir(proof_dir.as_ref(), "proof");
    proof_path.push(proof_name);
    proof_path.set_extension(PROOF_EXT);

    write_to_file(hex::encode(proof).as_bytes(), &proof_path);

    Ok(proof_path)
}
