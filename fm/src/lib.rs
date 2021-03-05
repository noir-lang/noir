mod file_map;
pub use file_map::{File, FileId, FileMap};

pub mod util;
use std::path::{Path, PathBuf};
pub use util::*;

pub const FILE_EXTENSION: &'static str = "nr";

// XXX: Create a trait for file io
/// An enum to differentiate between the root file
/// which the compiler starts at, and the others.
/// This is so that submodules of the root, can live alongside the
/// root file as files.
pub enum FileType {
    Root,
    Normal,
}

#[derive(Debug)]
struct VirtualPath(PathBuf);

#[derive(Debug)]
pub struct FileManager {
    file_map: file_map::FileMap,
    paths: Vec<VirtualPath>,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            file_map: file_map::FileMap::new(),
            paths: Vec::new(),
        }
    }

    // XXX: Maybe use a AsRef<Path> here, for API ergonomics
    // XXX: Would it break any assumptions, if we returned the same FileId,
    // given the same file_path?
    pub fn add_file(&mut self, path_to_file: &PathBuf, file_type: FileType) -> Option<FileId> {
        // We expect the caller to ensure that the file is a valid noir file
        let ext = path_to_file
            .extension()
            .expect(&format!("{:?} does not have an extension", path_to_file));
        if ext != FILE_EXTENSION {
            return None;
        }

        let source = std::fs::read_to_string(&path_to_file).ok()?;

        let file_id = self.file_map.add_file(path_to_file.clone().into(), source);
        let path_to_file = virtualise_path(path_to_file, file_type);
        self.paths.push(path_to_file);

        Some(file_id)
    }

    pub fn fetch_file(&mut self, file_id: FileId) -> File {
        // Unwrap as we ensure that all file_id's map to a corresponding file in the file map
        self.file_map.get_file(file_id).unwrap()
    }
    fn path(&mut self, file_id: FileId) -> &Path {
        // Unchecked as we ensure that all file_ids are created by the file manager
        // So all file_ids will points to a corresponding path
        self.paths[file_id.as_usize()].0.as_path()
    }

    pub fn resolve_path(&mut self, anchor: FileId, mod_name: &str) -> Result<FileId, String> {
        let mut candidate_files = Vec::new();

        let dir = self.path(anchor).to_path_buf();

        candidate_files.push(
            dir.to_path_buf()
                .join(&format!("{}.{}", mod_name, FILE_EXTENSION)),
        );

        for candidate in candidate_files.iter() {
            if let Some(file_id) = self.add_file(candidate, FileType::Normal) {
                return Ok(file_id);
            }
        }

        Err(candidate_files
            .remove(0)
            .as_os_str()
            .to_str()
            .unwrap()
            .to_owned())
    }
}
/// Takes a path to a noir file. This will panic on paths to directories
/// Returns
/// For Normal filetypes, given "src/mod.nr" this method returns "src/mod"
/// For Root filetypes, given "src/mod.nr" this method returns "src"
fn virtualise_path(path: &PathBuf, file_type: FileType) -> VirtualPath {
    let mut path = path.clone();
    let path = match file_type {
        FileType::Root => {
            path.pop();
            path
        }
        FileType::Normal => {
            let base = path.parent().unwrap();
            let path_no_ext: PathBuf = path
                .file_stem()
                .expect("ice: this should have been the path to a file")
                .into();
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
