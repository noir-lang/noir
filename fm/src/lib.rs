mod file_map;
pub use file_map::{FileID, File};

pub mod util;
pub use util::*;

use std::path::PathBuf;

pub const LIB_FILE : &'static str = "lib";
pub const BIN_FILE : &'static str = "main"; // XXX: This will be changed to be the entry point and not a static name.
pub const FILE_EXTENSION : &'static str = "nr";
pub const MOD_FILE : &'static str = "mod";

pub struct FileManager {
    file_map : file_map::FileMap,
    paths : Vec<PathBuf>
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            file_map : file_map::FileMap::new(),
            paths : Vec::new(),
        }
    }

    pub fn add_file(&mut self, path_to_file : PathBuf) -> Option<FileID> { 

        // We expect the caller to ensure that the file is a valid noir file
        let ext = path_to_file.extension().unwrap();
        assert_eq!(ext, FILE_EXTENSION);

        let source = std::fs::read_to_string(&path_to_file).ok()?;

        let file_id = self.file_map.add_file(path_to_file.clone().into(), source);
        self.paths.push(path_to_file);

        Some(file_id)
    }

    pub fn fetch_file(&mut self, file_id : FileID) -> File {
        self.file_map.get_file(file_id).unwrap()
    }

}