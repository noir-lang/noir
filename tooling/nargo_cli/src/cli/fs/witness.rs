use std::path::{Path, PathBuf};

use acvm::acir::native_types::WitnessStack;
use nargo::constants::WITNESS_EXT;

use super::{create_named_dir, write_to_file};
use crate::errors::FilesystemError;

pub(crate) fn save_witness_to_dir<P: AsRef<Path>>(
    witness_stack: WitnessStack,
    witness_name: &str,
    witness_dir: P,
) -> Result<PathBuf, FilesystemError> {
    create_named_dir(witness_dir.as_ref(), "witness");
    let witness_path = witness_dir.as_ref().join(witness_name).with_extension(WITNESS_EXT);

    let buf: Vec<u8> = witness_stack.try_into()?;
    println!("{:#?}", buf);
    let bytes = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 237, 145, 177, 13, 0, 32, 8, 4, 17, 117, 31, 75, 75, 87,
        113, 255, 37, 44, 196, 5, 228, 42, 194, 39, 132, 238, 114, 249, 239, 114, 163, 118, 47,
        203, 254, 240, 101, 23, 152, 213, 120, 199, 73, 58, 42, 200, 170, 176, 87, 238, 27, 119,
        95, 201, 238, 190, 89, 7, 37, 195, 196, 176, 4, 5, 0, 0,
    ];
    write_to_file(buf.as_slice(), &witness_path);

    Ok(witness_path)
}
