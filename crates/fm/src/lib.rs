mod file_map;
mod file_reader;

pub use file_map::{File, FileId, FileMap};

pub mod util;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
pub use util::*;

pub const FILE_EXTENSION: &str = "nr";

// XXX: Create a trait for file io
/// An enum to differentiate between the root file
/// which the compiler starts at, and the others.
/// This is so that submodules of the root, can live alongside the
/// root file as files.
pub enum FileType {
    Root,
    Normal,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct VirtualPath(PathBuf);

#[derive(Debug)]
pub struct FileManager {
    file_map: file_map::FileMap,
    id_to_path: HashMap<FileId, VirtualPath>,
    path_to_id: HashMap<VirtualPath, FileId>,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            file_map: file_map::FileMap::new(),
            id_to_path: HashMap::new(),
            path_to_id: HashMap::new(),
        }
    }

    // XXX: Maybe use a AsRef<Path> here, for API ergonomics
    pub fn add_file(&mut self, path_to_file: &Path, file_type: FileType) -> Option<FileId> {
        let source = file_reader::read_file_to_string(path_to_file).ok()?;
        self.add_file_with_source(path_to_file, source, file_type)
    }

    // This is only used in the wasm context, where we want to add the stdlib
    // with only its source file.
    // We add a file path which won't accidentally be the filepath to a user file
    // We use a uuid to ensure that this is not the case.
    pub fn add_file_with_dummy_path(
        &mut self,
        source: String,
        file_type: FileType,
    ) -> Option<FileId> {
        let file_path = PathBuf::from(uuid::Uuid::new_v4().to_string());

        self.add_file_with_source(file_path.as_path(), source, file_type)
    }

    pub(crate) fn add_file_with_source(
        &mut self,
        path_to_file: &Path,
        source: String,
        file_type: FileType,
    ) -> Option<FileId> {
        // We expect the caller to ensure that the file is a valid noir file
        let ext = path_to_file
            .extension()
            .unwrap_or_else(|| panic!("{:?} does not have an extension", path_to_file));
        if ext != FILE_EXTENSION {
            return None;
        }

        let file_id = self.file_map.add_file(path_to_file.to_path_buf().into(), source);
        let path_to_file = virtualise_path(path_to_file, file_type);
        self.register_path(file_id, path_to_file);

        Some(file_id)
    }

    fn register_path(&mut self, file_id: FileId, path: VirtualPath) {
        let old_value = self.id_to_path.insert(file_id, path.clone());
        assert!(
            old_value.is_none(),
            "ice: the same file id was inserted into the file manager twice"
        );
        let old_value = self.path_to_id.insert(path, file_id);
        assert!(old_value.is_none(), "ice: the same path was inserted into the file manager twice");
    }

    pub fn fetch_file(&mut self, file_id: FileId) -> File {
        // Unwrap as we ensure that all file_id's map to a corresponding file in the file map
        self.file_map.get_file(file_id).unwrap()
    }
    fn path(&mut self, file_id: FileId) -> &Path {
        // Unwrap as we ensure that all file_ids are created by the file manager
        // So all file_ids will points to a corresponding path
        self.id_to_path.get(&file_id).unwrap().0.as_path()
    }

    pub fn resolve_path(&mut self, anchor: FileId, mod_name: &str) -> Result<FileId, String> {
        let mut candidate_files = Vec::new();

        let dir = self.path(anchor).to_path_buf();

        candidate_files.push(dir.join(&format!("{}.{}", mod_name, FILE_EXTENSION)));

        for candidate in candidate_files.iter() {
            if let Some(file_id) = self.add_file(candidate, FileType::Normal) {
                return Ok(file_id);
            }
        }

        Err(candidate_files.remove(0).as_os_str().to_str().unwrap().to_owned())
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Takes a path to a noir file. This will panic on paths to directories
/// Returns
/// For Normal filetypes, given "src/mod.nr" this method returns "src/mod"
/// For Root filetypes, given "src/mod.nr" this method returns "src"
fn virtualise_path(path: &Path, file_type: FileType) -> VirtualPath {
    let mut path = path.to_path_buf();
    let path = match file_type {
        FileType::Root => {
            path.pop();
            path
        }
        FileType::Normal => {
            let base = path.parent().unwrap();
            let path_no_ext: PathBuf =
                path.file_stem().expect("ice: this should have been the path to a file").into();
            base.join(path_no_ext)
        }
    };
    VirtualPath(path)
}
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, TempDir};

    fn dummy_file_path(dir: &TempDir, file_name: &str) -> PathBuf {
        let file_path = dir.path().join(file_name);
        let _file = std::fs::File::create(file_path.clone()).unwrap();

        file_path
    }

    #[test]
    fn path_resolve_file_module() {
        let dir = tempdir().unwrap();
        let file_path = dummy_file_path(&dir, "my_dummy_file.nr");

        let mut fm = FileManager::new();

        let file_id = fm.add_file(&file_path, FileType::Root).unwrap();

        let _foo_file_path = dummy_file_path(&dir, "foo.nr");
        fm.resolve_path(file_id, "foo").unwrap();
    }
    #[test]
    fn path_resolve_sub_module() {
        let mut fm = FileManager::new();

        let dir = tempdir().unwrap();
        // Create a lib.nr file at the root.
        // we now have dir/lib.nr
        let file_path = dummy_file_path(&dir, "lib.nr");

        let file_id = fm.add_file(&file_path, FileType::Root).unwrap();

        // Create a sub directory
        // we now have:
        // - dir/lib.nr
        // - dir/sub_dir
        let sub_dir = TempDir::new_in(&dir).unwrap();
        std::fs::create_dir_all(sub_dir.path()).unwrap();
        let sub_dir_name = sub_dir.path().file_name().unwrap().to_str().unwrap();

        // Add foo.nr to the subdirectory
        // we no have:
        // - dir/lib.nr
        // - dir/sub_dir/foo.nr
        let _foo_file_path = dummy_file_path(&sub_dir, "foo.nr");

        // Add a parent module for the sub_dir
        // we no have:
        // - dir/lib.nr
        // - dir/sub_dir.nr
        // - dir/sub_dir/foo.nr
        let _sub_dir_root_file_path = dummy_file_path(&dir, &format!("{}.nr", sub_dir_name));

        // First check for the sub_dir.nr file and add it to the FileManager
        let sub_dir_file_id = fm.resolve_path(file_id, sub_dir_name).unwrap();

        // Now check for files in it's subdirectory
        fm.resolve_path(sub_dir_file_id, "foo").unwrap();
    }
}
