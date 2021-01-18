mod file_map;
pub use file_map::{FileId, File, FileMap};

pub mod util;
pub use util::*;

use std::path::{Path, PathBuf};

pub const LIB_FILE : &'static str = "lib";
pub const BIN_FILE : &'static str = "main"; // XXX: This will be changed to be the entry point and not a static name.
pub const FILE_EXTENSION : &'static str = "nr";
pub const MOD_FILE : &'static str = "mod";

// XXX: Create aa trait for file io

#[derive(Debug)]
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

    // XXX: Maybe use a AsRef<Path> here, for API ergonomics
    pub fn add_file(&mut self, path_to_file : &PathBuf) -> Option<FileId> { 

        // We expect the caller to ensure that the file is a valid noir file
        let ext = path_to_file.extension().expect(&format!("{:?} does not have an extension", path_to_file));
        if ext != FILE_EXTENSION {
            return None
        }

        let source = std::fs::read_to_string(&path_to_file).ok()?;

        let file_id = self.file_map.add_file(path_to_file.clone().into(), source);
        self.paths.push(path_to_file.clone());

        Some(file_id)
    }

    pub fn fetch_file(&mut self, file_id : FileId) -> File {
        // Unwrap as we ensure that all file_id's map to a corresponding file in the file map
        self.file_map.get_file(file_id).unwrap()
    }
    pub fn path(&mut self, file_id : FileId) -> &Path {
        // Unchecked as we ensure that all file_ids are created by the file manager 
        // So all file_ids will points to a corresponding path
        self.paths[file_id.as_usize()].as_path()
    }
    pub fn parent(&mut self, file_id : FileId) -> &Path {
        // Unwrap as we ensure that all file_ids's point to files
        // whom logically live in some directory
        self.path(file_id).parent().unwrap()
    }

    pub fn resolve_path(&mut self, anchor : FileId, mod_name : &str) -> Result<FileId, String> {

        let mut candidate_files = Vec::new();

        let dir = self.parent(anchor).to_path_buf();

        candidate_files.push(dir.to_path_buf().join(&format!("{}.nr", mod_name)));
        candidate_files.push(dir.to_path_buf().join(&format!("{}/mod.nr", mod_name)));

        for candidate in candidate_files.iter() {
            if let Some(file_id) = self.add_file(candidate) {
                return Ok(file_id)
            }
        }

        Err(candidate_files.remove(0).as_os_str().to_str().unwrap().to_owned())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::{TempDir, tempdir};
    use super::*;
    
    fn dummy_file_path(dir : &TempDir, file_name : &str) -> PathBuf {
        
        let file_path = dir.path().join(file_name);
        let _file = std::fs::File::create(file_path.clone()).unwrap();
        
        file_path
    }
    
    #[test]
    fn path_resolve_file_module() {
        
        let dir = tempdir().unwrap();
        let file_path = dummy_file_path(&dir, "my_dummy_file.nr");
        
        let mut fm = FileManager::new();
        
        let file_id = fm.add_file(&file_path).unwrap();
        
        let _foo_file_path = dummy_file_path(&dir, "foo.nr");
        fm.resolve_path(file_id, "foo").unwrap();
}
    #[test]
    fn path_resolve_sub_module() {
        let mut fm = FileManager::new();
        
        let dir = tempdir().unwrap();
        let file_path = dummy_file_path(&dir, "my_dummy_file.nr");
        
        let file_id = fm.add_file(&file_path).unwrap();

        let sub_dir = TempDir::new_in(dir).unwrap();
        std::fs::create_dir_all(sub_dir.path()).unwrap();
        let tmp_dir_name = sub_dir.path().file_name().unwrap().to_str().unwrap();

        let _foo_file_path = dummy_file_path(&sub_dir, "mod.nr");
        fm.resolve_path(file_id, tmp_dir_name).unwrap();
    }
}