#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

mod file_map;
mod file_reader;

pub use file_map::{File, FileId, FileMap};
use file_reader::is_stdlib_asset;

use std::{
    collections::HashMap,
    path::{Component, Path, PathBuf},
};

pub const FILE_EXTENSION: &str = "nr";

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct VirtualPath(PathBuf);

#[derive(Debug)]
pub struct FileManager {
    root: PathBuf,
    file_map: file_map::FileMap,
    id_to_path: HashMap<FileId, VirtualPath>,
    path_to_id: HashMap<VirtualPath, FileId>,
}

impl FileManager {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.normalize(),
            file_map: Default::default(),
            id_to_path: Default::default(),
            path_to_id: Default::default(),
        }
    }

    pub fn add_file(&mut self, file_name: &Path) -> Option<FileId> {
        // Handle both relative file paths and std/lib virtual paths.
        let resolved_path: PathBuf = if is_stdlib_asset(file_name) {
            // Special case for stdlib where we want to read specifically the `std/` relative path
            // TODO: The stdlib path should probably be an absolute path rooted in something people would never create
            file_name.to_path_buf()
        } else {
            self.root.join(file_name).normalize()
        };

        // Check that the resolved path already exists in the file map, if it is, we return it.
        let path_to_file = virtualize_path(&resolved_path);
        if let Some(file_id) = self.path_to_id.get(&path_to_file) {
            return Some(*file_id);
        }

        // Otherwise we add the file
        let source = file_reader::read_file_to_string(&resolved_path).ok()?;
        let file_id = self.file_map.add_file(resolved_path.into(), source);
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

    pub fn find_module(&mut self, anchor: FileId, mod_name: &str) -> Result<FileId, String> {
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

pub trait NormalizePath {
    /// Replacement for `std::fs::canonicalize` that doesn't verify the path exists.
    ///
    /// Plucked from https://github.com/rust-lang/cargo/blob/fede83ccf973457de319ba6fa0e36ead454d2e20/src/cargo/util/paths.rs#L61
    /// Advice from https://www.reddit.com/r/rust/comments/hkkquy/comment/fwtw53s/
    fn normalize(&self) -> PathBuf;
}

impl NormalizePath for PathBuf {
    fn normalize(&self) -> PathBuf {
        let components = self.components();
        resolve_components(components)
    }
}

impl NormalizePath for &Path {
    fn normalize(&self) -> PathBuf {
        let components = self.components();
        resolve_components(components)
    }
}

fn resolve_components<'a>(components: impl Iterator<Item = Component<'a>>) -> PathBuf {
    let mut components = components.peekable();

    // Preserve path prefix if one exists.
    let mut normalized_path = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!("Path cannot contain multiple prefixes"),
            Component::RootDir => {
                normalized_path.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                normalized_path.pop();
            }
            Component::Normal(c) => {
                normalized_path.push(c);
            }
        }
    }

    normalized_path
}

#[cfg(test)]
mod path_normalization {
    use iter_extended::vecmap;
    use std::path::PathBuf;

    use crate::NormalizePath;

    #[test]
    fn normalizes_paths_correctly() {
        // Note that tests are run on unix so prefix handling can't be tested (as these only exist on Windows)
        let test_cases = vecmap(
            [
                ("/", "/"),                             // Handles root
                ("/foo/bar/../baz/../bar", "/foo/bar"), // Handles backtracking
                ("/././././././././baz", "/baz"),       // Removes no-ops
            ],
            |(unnormalized, normalized)| (PathBuf::from(unnormalized), PathBuf::from(normalized)),
        );

        for (path, expected_result) in test_cases {
            assert_eq!(path.normalize(), expected_result);
        }
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

    fn create_dummy_file(dir: &TempDir, file_name: &Path) {
        let file_path = dir.path().join(file_name);
        let _file = std::fs::File::create(file_path).unwrap();
    }

    #[test]
    fn path_resolve_file_module() {
        let dir = tempdir().unwrap();

        let entry_file_name = Path::new("my_dummy_file.nr");
        create_dummy_file(&dir, entry_file_name);

        let mut fm = FileManager::new(dir.path());

        let file_id = fm.add_file(entry_file_name).unwrap();

        let dep_file_name = Path::new("foo.nr");
        create_dummy_file(&dir, dep_file_name);
        fm.find_module(file_id, "foo").unwrap();
    }
    #[test]
    fn path_resolve_file_module_other_ext() {
        let dir = tempdir().unwrap();
        let file_name = Path::new("foo.noir");
        create_dummy_file(&dir, file_name);

        let mut fm = FileManager::new(dir.path());

        let file_id = fm.add_file(file_name).unwrap();

        assert!(fm.path(file_id).ends_with("foo"));
    }
    #[test]
    fn path_resolve_sub_module() {
        let dir = tempdir().unwrap();
        let mut fm = FileManager::new(dir.path());

        // Create a lib.nr file at the root.
        // we now have dir/lib.nr
        let file_name = Path::new("lib.nr");
        create_dummy_file(&dir, file_name);

        let file_id = fm.add_file(file_name).unwrap();

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
        create_dummy_file(&sub_dir, Path::new("foo.nr"));

        // Add a parent module for the sub_dir
        // we no have:
        // - dir/lib.nr
        // - dir/sub_dir.nr
        // - dir/sub_dir/foo.nr
        create_dummy_file(&dir, Path::new(&format!("{}.nr", sub_dir_name)));

        // First check for the sub_dir.nr file and add it to the FileManager
        let sub_dir_file_id = fm.find_module(file_id, sub_dir_name).unwrap();

        // Now check for files in it's subdirectory
        fm.find_module(sub_dir_file_id, "foo").unwrap();
    }

    /// Tests that two identical files that have different paths are treated as the same file
    /// e.g. if we start in the dir ./src and have a file ../../foo.nr
    /// that should be treated as the same file as ../ starting in ./
    /// they should both resolve to ../foo.nr
    #[test]
    fn path_resolve_modules_with_different_paths_as_same_file() {
        let dir = tempdir().unwrap();
        let sub_dir = TempDir::new_in(&dir).unwrap();
        let sub_sub_dir = TempDir::new_in(&sub_dir).unwrap();

        let mut fm = FileManager::new(dir.path());

        // Create a lib.nr file at the root.
        let file_name = Path::new("lib.nr");
        create_dummy_file(&dir, file_name);

        // Create another path with `./` and `../` inside it
        let second_file_name = PathBuf::from(sub_sub_dir.path()).join("./../../lib.nr");

        // Add both files to the file manager
        let file_id = fm.add_file(file_name).unwrap();
        let second_file_id = fm.add_file(&second_file_name).unwrap();

        assert_eq!(file_id, second_file_id);
    }
}
