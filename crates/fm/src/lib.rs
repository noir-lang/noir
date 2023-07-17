#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

mod file_map;
mod file_reader;

pub use file_map::{File, FileId, FileMap};

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub const FILE_EXTENSION: &str = "nr";

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct VirtualPath(PathBuf);

#[derive(Debug, Default)]
pub struct FileManager {
    file_map: file_map::FileMap,
    id_to_path: HashMap<FileId, VirtualPath>,
    path_to_id: HashMap<VirtualPath, FileId>,
}

impl FileManager {
    // XXX: Maybe use a AsRef<Path> here, for API ergonomics
    pub fn add_file(&mut self, path_to_file: &Path) -> Option<FileId> {
        // Handle both relative file paths and std/lib virtual paths.
        let base = Path::new(".").canonicalize().expect("Base path canonicalize failed");
        let res = path_to_file.canonicalize().unwrap_or_else(|_| path_to_file.to_path_buf());
        let resolved_path = res.strip_prefix(base).unwrap_or(&res);

        // Check that the resolved path already exists in the file map, if it is, we return it.
        let path_to_file = virtualize_path(resolved_path);
        if let Some(file_id) = self.path_to_id.get(&path_to_file) {
            return Some(*file_id);
        }

        // Otherwise we add the file
        let source = file_reader::read_file_to_string(resolved_path).ok()?;
        let file_id = self.file_map.add_file(resolved_path.to_path_buf().into(), source);
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
    pub fn path(&self, file_id: FileId) -> &Path {
        // Unwrap as we ensure that all file_ids are created by the file manager
        // So all file_ids will points to a corresponding path
        self.id_to_path.get(&file_id).unwrap().0.as_path()
    }

    pub fn resolve_path(&mut self, anchor: FileId, mod_name: &str) -> Result<FileId, String> {
        let mut candidate_files = Vec::new();

        let anchor_path = self.path(anchor).to_path_buf();
        let anchor_dir = anchor_path.parent().unwrap();

        // First we attempt to look at `base/anchor/mod_name.nr` (child of the anchor)
        candidate_files.push(anchor_path.join(format!("{mod_name}.{FILE_EXTENSION}")));
        // If not found, we attempt to look at `base/mod_name.nr` (sibling of the anchor)
        candidate_files.push(anchor_dir.join(format!("{mod_name}.{FILE_EXTENSION}")));

        for candidate in candidate_files.iter() {
            if let Some(file_id) = self.add_file(candidate) {
                return Ok(file_id);
            }
        }

        Err(candidate_files.remove(0).as_os_str().to_str().unwrap().to_owned())
    }
}

/// Takes a path to a noir file. This will panic on paths to directories
/// Returns the file path with the extension removed
fn virtualize_path(path: &Path) -> VirtualPath {
    let path = path.to_path_buf();
    let base = path.parent().unwrap();
    let path_no_ext: PathBuf =
        path.file_stem().expect("ice: this should have been the path to a file").into();
    let path = base.join(path_no_ext);
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

        let mut fm = FileManager::default();

        let file_id = fm.add_file(&file_path).unwrap();

        let _foo_file_path = dummy_file_path(&dir, "foo.nr");
        fm.resolve_path(file_id, "foo").unwrap();
    }
    #[test]
    fn path_resolve_file_module_other_ext() {
        let dir = tempdir().unwrap();
        let file_path = dummy_file_path(&dir, "foo.noir");

        let mut fm = FileManager::default();

        let file_id = fm.add_file(&file_path).unwrap();

        assert!(fm.path(file_id).ends_with("foo"));
    }
    #[test]
    fn path_resolve_sub_module() {
        let mut fm = FileManager::default();

        let dir = tempdir().unwrap();
        // Create a lib.nr file at the root.
        // we now have dir/lib.nr
        let file_path = dummy_file_path(&dir, "lib.nr");

        let file_id = fm.add_file(&file_path).unwrap();

        // Create a sub directory
        // we now have:
        // - dir/lib.nr
        // - dir/sub_dir
        let sub_dir = TempDir::new_in(&dir).unwrap();
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

    /// Tests that two identical files that have different paths are treated as the same file
    /// e.g. if we start in the dir ./src and have a file ../../foo.nr
    /// that should be treated as the same file as ../ starting in ./
    /// they should both resolve to ../foo.nr
    #[test]
    fn path_resolve_modules_with_different_paths_as_same_file() {
        let mut fm = FileManager::default();

        // Create a lib.nr file at the root.
        let dir = tempdir().unwrap();
        let sub_dir = TempDir::new_in(&dir).unwrap();
        let sub_sub_dir = TempDir::new_in(&sub_dir).unwrap();
        let file_path = dummy_file_path(&dir, "lib.nr");

        // Create another file in a subdirectory with a convoluted path
        let second_file_path = dummy_file_path(&sub_sub_dir, "./../../lib.nr");

        // Add both files to the file manager
        let file_id = fm.add_file(&file_path).unwrap();
        let second_file_id = fm.add_file(&second_file_path).unwrap();

        assert_eq!(file_id, second_file_id);
    }
}
