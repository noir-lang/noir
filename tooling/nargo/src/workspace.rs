// We will say that a cargo unit must contain either a binary or a library
// Then we use workspace to allow more than one. In the future, do not allow there to be
// both a binary and a library.
// - library will be default

use std::{
    iter::{once, Once},
    path::PathBuf,
    slice,
};

use crate::{
    constants::{CONTRACT_DIR, EXPORT_DIR, PROOFS_DIR, TARGET_DIR},
    package::Package,
};

#[derive(Clone)]
pub struct Workspace {
    pub root_dir: PathBuf,
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

    pub fn contracts_directory_path(&self, package: &Package) -> PathBuf {
        let name: String = package.name.clone().into();
        self.root_dir.join(CONTRACT_DIR).join(name)
    }

    pub fn proofs_directory_path(&self) -> PathBuf {
        self.root_dir.join(PROOFS_DIR)
    }

    pub fn target_directory_path(&self) -> PathBuf {
        self.root_dir.join(TARGET_DIR)
    }

    pub fn export_directory_path(&self) -> PathBuf {
        self.root_dir.join(EXPORT_DIR)
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
