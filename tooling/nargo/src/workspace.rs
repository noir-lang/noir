// We will say that a cargo unit must contain either a binary or a library
// Then we use workspace to allow more than one. In the future, do not allow there to be
// both a binary and a library.
// - library will be default

use std::{
    iter::{once, Once},
    path::PathBuf,
    slice,
};

use fm::FileManager;
use noirc_driver::file_manager_with_stdlib;

use crate::{
    constants::{EXPORT_DIR, TARGET_DIR},
    package::Package,
};

#[derive(Clone)]
pub struct Workspace {
    pub root_dir: PathBuf,
    /// Optional target directory override.
    pub target_dir: Option<PathBuf>,
    pub members: Vec<Package>,
    // If `Some()`, the `selected_package_index` is used to select the only `Package` when iterating a Workspace
    pub selected_package_index: Option<usize>,
    /// If we could not resolve the workspace we would inform the user we have assumed it (ie. from lsp file path given)
    pub is_assumed: bool,
}

impl Workspace {
    pub fn package_build_path(&self, package: &Package) -> PathBuf {
        let name: String = package.name.clone().into();
        self.target_directory_path().join(name).with_extension("json")
    }

    pub fn target_directory_path(&self) -> PathBuf {
        self.target_dir.as_ref().cloned().unwrap_or_else(|| self.root_dir.join(TARGET_DIR))
    }

    pub fn export_directory_path(&self) -> PathBuf {
        self.root_dir.join(EXPORT_DIR)
    }

    /// Returns a new `FileManager` for the root directory of this workspace.
    /// If the root directory is not the standard library, the standard library
    /// is added to the returned `FileManager`.
    pub fn new_file_manager(&self) -> FileManager {
        if self.is_stdlib() {
            FileManager::new(&self.root_dir)
        } else {
            file_manager_with_stdlib(&self.root_dir)
        }
    }

    fn is_stdlib(&self) -> bool {
        self.members.len() == 1 && self.members[0].name.to_string() == "std"
    }
}

pub enum IntoIter<'a, T> {
    Only(Once<&'a T>),
    All(slice::Iter<'a, T>),
}

impl<'a> IntoIterator for &'a Workspace {
    type Item = &'a Package;
    type IntoIter = IntoIter<'a, Package>;

    fn into_iter(self) -> Self::IntoIter {
        if let Some(index) = self.selected_package_index {
            // Precondition: The selected_package_index was verified to be in-bounds before constructing workspace
            let member = self
                .members
                .get(index)
                .expect("Workspace constructed with invalid selected_package_index");

            IntoIter::Only(once(member))
        } else {
            IntoIter::All(self.members.iter())
        }
    }
}

impl<'a> Iterator for IntoIter<'a, Package> {
    type Item = &'a Package;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Only(iter) => iter.next(),
            Self::All(iter) => iter.next(),
        }
    }
}
