#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

mod file_map;

pub use file_map::{File, FileId, FileMap, PathString};

// Re-export for the lsp
pub use codespan_reporting::files as codespan_files;

use std::{
    collections::HashMap,
    path::{Component, Path, PathBuf},
};

pub const FILE_EXTENSION: &str = "nr";
#[derive(Clone)]
pub struct FileManager {
    root: PathBuf,
    file_map: FileMap,
    id_to_path: HashMap<FileId, PathBuf>,
    path_to_id: HashMap<PathBuf, FileId>,
}

impl std::fmt::Debug for FileManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileManager")
            .field("root", &self.root)
            .field("file_map", &self.file_map)
            .field("id_to_path", &self.id_to_path)
            .field("path_to_id", &self.path_to_id)
            .finish()
    }
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

    pub fn as_file_map(&self) -> &FileMap {
        &self.file_map
    }

    /// Adds a source file to the [`FileManager`].
    ///
    /// The `file_name` is expected to be relative to the [`FileManager`]'s root directory.
    pub fn add_file_with_source(&mut self, file_name: &Path, source: String) -> Option<FileId> {
        let file_name = self.root.join(file_name);
        self.add_file_with_source_canonical_path(&file_name, source)
    }

    /// Adds a source file to the [`FileManager`] using a path which is not appended to the root path.
    ///
    /// This should only be used for the stdlib as these files do not exist on the user's filesystem.
    pub fn add_file_with_source_canonical_path(
        &mut self,
        file_name: &Path,
        source: String,
    ) -> Option<FileId> {
        let file_name = file_name.normalize();
        // Check that the file name already exists in the file map, if it is, we return it.
        if let Some(file_id) = self.path_to_id.get(&file_name) {
            return Some(*file_id);
        }
        let file_name_path_buf = file_name.to_path_buf();

        // Otherwise we add the file
        let file_id = self.file_map.add_file(file_name_path_buf.clone().into(), source);
        self.register_path(file_id, file_name_path_buf);
        Some(file_id)
    }

    fn register_path(&mut self, file_id: FileId, path: PathBuf) {
        let old_value = self.id_to_path.insert(file_id, path.clone());
        assert!(
            old_value.is_none(),
            "ice: the same file id was inserted into the file manager twice"
        );
        let old_value = self.path_to_id.insert(path, file_id);
        assert!(old_value.is_none(), "ice: the same path was inserted into the file manager twice");
    }

    pub fn fetch_file(&self, file_id: FileId) -> Option<&str> {
        // Unwrap as we ensure that all file_id's map to a corresponding file in the file map
        self.file_map.get_file(file_id).map(|file| file.source())
    }

    pub fn path(&self, file_id: FileId) -> Option<&Path> {
        // Unwrap as we ensure that all file_ids are created by the file manager
        // So all file_ids will points to a corresponding path
        self.id_to_path.get(&file_id).map(|path| path.as_path())
    }

    // TODO: This should accept a &Path instead of a PathBuf
    pub fn name_to_id(&self, file_name: PathBuf) -> Option<FileId> {
        self.file_map.get_file_id(&PathString::from_path(file_name))
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, TempDir};

    // Returns the absolute path to the file
    fn create_dummy_file(dir: &TempDir, file_name: &Path) -> PathBuf {
        let file_path = dir.path().join(file_name);
        let _file = std::fs::File::create(&file_path).unwrap();
        file_path
    }

    #[test]
    fn path_resolve_file_module_other_ext() {
        let dir = tempdir().unwrap();
        let file_name = Path::new("foo.nr");
        create_dummy_file(&dir, file_name);

        let mut fm = FileManager::new(dir.path());

        let file_id = fm.add_file_with_source(file_name, "fn foo() {}".to_string()).unwrap();

        assert!(fm.path(file_id).unwrap().ends_with("foo.nr"));
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
        let file_id = fm.add_file_with_source(file_name, "fn foo() {}".to_string()).unwrap();
        let second_file_id =
            fm.add_file_with_source(&second_file_name, "fn foo() {}".to_string()).unwrap();

        assert_eq!(file_id, second_file_id);
    }
}
