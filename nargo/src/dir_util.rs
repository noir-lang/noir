use std::path::{Path, PathBuf};
use std::fs::ReadDir;

pub const LIB_FILE : &'static str = "lib";
pub const MAIN_FILE : &'static str = "main";
pub const FILE_EXTENSION : &'static str = "nr";
pub const MOD_FILE : &'static str = "mod";

pub enum DirType {
    Library,
    Binary,
    Module
}

impl DirType {
    fn file_name(&self) -> &'static str {
        match self {
            DirType::Binary => MAIN_FILE,
            DirType::Library => LIB_FILE,
            DirType::Module => MOD_FILE,
        }
    }

    /// Return None, if the DirType file is not present and Some if it is
    pub fn file_path<P: AsRef<Path>>(&self, path : P) -> Option<PathBuf> {
        find_file(path, self.file_name(), FILE_EXTENSION)
    }
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