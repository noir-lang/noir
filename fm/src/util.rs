use std::path::{Path, PathBuf};
use std::fs::ReadDir;
use crate::{MOD_FILE, LIB_FILE, BIN_FILE, FILE_EXTENSION};


pub fn find_mod_file<P: AsRef<Path>>(path : P) -> Option<PathBuf>{
    file_path(MOD_FILE, path)
}
pub fn find_lib_file<P: AsRef<Path>>(path : P) -> Option<PathBuf>{
    file_path(LIB_FILE, path)
    
}
pub fn find_bin_file<P: AsRef<Path>>(path : P) -> Option<PathBuf>{
    file_path(BIN_FILE, path)

}

/// Return None, if the file with extension is not present and Some if it is
pub fn file_path<P: AsRef<Path>>(file_name : &str, path : P) -> Option<PathBuf> {
    find_file(path, file_name, FILE_EXTENSION)
}

// Looks for file named `file_name` in path
pub fn find_file<P: AsRef<Path>>(path : P, file_name : &str, extension : &str) -> Option<PathBuf> {
    let entries = list_files_and_folders_in(path)?;
    
    let mut file_name = file_name.to_owned();
    file_name.push_str(".");
    file_name.push_str(extension);

    find_artifact(entries, &file_name)
}
// Looks for directory named `dir_name` in path
pub fn find_dir<P: AsRef<Path>>(path : P, dir_name : &str) -> Option<PathBuf> {
    let entries = list_files_and_folders_in(path)?;
    find_artifact(entries, dir_name)
}

// There is no distinction between files and folders
fn find_artifact(entries : ReadDir, artifact_name : &str) -> Option<PathBuf> {
    let mut entry : Vec<_> = entries.into_iter()
    .flatten()
    .filter(|entry| entry.file_name().to_str() == Some(artifact_name)).collect();

    Some(entry.pop()?.path())
}

fn list_files_and_folders_in<P: AsRef<Path>>(path : P) -> Option<ReadDir> {
    std::fs::read_dir(path).ok()
}