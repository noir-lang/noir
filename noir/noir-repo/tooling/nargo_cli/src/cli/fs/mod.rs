use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use crate::errors::FilesystemError;

pub(super) mod inputs;
pub(super) mod program;
pub(super) mod proof;
pub(super) mod witness;

pub(super) fn create_named_dir(named_dir: &Path, name: &str) -> PathBuf {
    std::fs::create_dir_all(named_dir)
        .unwrap_or_else(|_| panic!("could not create the `{name}` directory"));

    PathBuf::from(named_dir)
}

pub(super) fn write_to_file(bytes: &[u8], path: &Path) -> String {
    let display = path.display();

    let mut file = match File::create(path) {
        Err(why) => panic!("couldn't create {display}: {why}"),
        Ok(file) => file,
    };

    match file.write_all(bytes) {
        Err(why) => panic!("couldn't write to {display}: {why}"),
        Ok(_) => display.to_string(),
    }
}

pub(super) fn load_hex_data<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, FilesystemError> {
    let hex_data: Vec<_> = std::fs::read(&path)
        .map_err(|_| FilesystemError::PathNotValid(path.as_ref().to_path_buf()))?;

    let raw_bytes = hex::decode(hex_data).map_err(FilesystemError::HexArtifactNotValid)?;

    Ok(raw_bytes)
}
