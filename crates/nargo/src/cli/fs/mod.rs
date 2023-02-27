use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use crate::errors::CliError;

pub(super) mod acir;
pub(super) mod inputs;
pub(super) mod keys;
pub(super) mod proof;
pub(super) mod witness;

fn create_dir<P: AsRef<Path>>(dir_path: P) -> Result<PathBuf, std::io::Error> {
    let mut dir = std::path::PathBuf::new();
    dir.push(dir_path);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub(super) fn create_named_dir(named_dir: &Path, name: &str) -> PathBuf {
    create_dir(named_dir).unwrap_or_else(|_| panic!("could not create the `{name}` directory"))
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

pub(super) fn load_hex_data<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, CliError> {
    let hex_data: Vec<_> =
        std::fs::read(&path).map_err(|_| CliError::PathNotValid(path.as_ref().to_path_buf()))?;

    let raw_bytes = hex::decode(hex_data).map_err(CliError::HexArtifactNotValid)?;

    Ok(raw_bytes)
}
