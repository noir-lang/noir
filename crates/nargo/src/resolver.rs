use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use acvm::Language;
use noirc_driver::Driver;
use noirc_frontend::graph::{CrateId, CrateName, CrateType};
use thiserror::Error;

use crate::{
    git::clone_git_repo,
    manifest::{Dependency, PackageManifest},
    InvalidPackageError,
};

/// Creates a unique folder name for a GitHub repo
/// by using it's URL and tag
pub(crate) fn resolve_folder_name(base: &url::Url, tag: &str) -> String {
    let mut folder_name = base.domain().unwrap().to_owned();
    folder_name.push_str(base.path());
    folder_name.push_str(tag);
    folder_name
}

/// Errors covering situations where a crate's dependency tree cannot be resolved.
#[derive(Debug, Error)]
pub(crate) enum DependencyResolutionError {
    /// Encountered error while downloading git repository.
    #[error("{0}")]
    GitError(String),

    /// Attempted to depend on a binary crate.
    #[error("dependency {dep_pkg_name} is a binary package and so it cannot be depended upon.")]
    BinaryDependency { dep_pkg_name: String },

    /// Attempted to depend on remote crate which has a local dependency.
    /// We have no guarantees that this local dependency will be available so must error.
    #[error("remote(git) dependency has a local dependency.\ndependency located at {}", dependency_path.display())]
    RemoteDepWithLocalDep { dependency_path: PathBuf },

    /// Dependency is not a valid crate
    #[error(transparent)]
    MalformedDependency(#[from] InvalidPackageError),
}

#[derive(Debug, Clone)]
struct CachedDep {
    entry_path: PathBuf,
    crate_type: CrateType,
    manifest: PackageManifest,
    // Whether the dependency came from
    // a remote dependency
    remote: bool,
}

/// Resolves a toml file by either downloading the necessary git repo
/// or it uses the repo on the cache.
/// Downloading will be recursive, so if a package contains packages
/// We need to download those too
pub(crate) struct Resolver<'a> {
    cached_packages: HashMap<PathBuf, (CrateId, CachedDep)>,
    driver: &'a mut Driver,
}

impl<'a> Resolver<'a> {
    fn with_driver(driver: &mut Driver) -> Resolver {
        Resolver { cached_packages: HashMap::new(), driver }
    }

    /// Returns the Driver and the backend to use
    /// Note that the backend is ignored in the dependencies.
    /// Since Noir is backend agnostic, this is okay to do.
    /// XXX: Need to handle when a local package changes!
    pub(crate) fn resolve_root_manifest(
        dir_path: &std::path::Path,
        np_language: Language,
    ) -> Result<Driver, DependencyResolutionError> {
        let mut driver = Driver::new(&np_language);
        let (entry_path, crate_type) = super::lib_or_bin(dir_path)?;

        let manifest_path = super::find_package_manifest(dir_path)?;
        let manifest = super::manifest::parse(manifest_path)?;

        let crate_id = driver.create_local_crate(entry_path, crate_type);

        let mut resolver = Resolver::with_driver(&mut driver);
        resolver.resolve_manifest(crate_id, manifest)?;

        add_std_lib(&mut driver);
        Ok(driver)
    }

    // Resolves a config file by recursively resolving the dependencies in the config
    // Need to solve the case of a project trying to use itself as a dep
    //
    // We do not need to add stdlib, as it's implicitly
    // imported. However, it may be helpful to have the stdlib imported by the
    // package manager.
    fn resolve_manifest(
        &mut self,
        parent_crate: CrateId,
        manifest: PackageManifest,
    ) -> Result<(), DependencyResolutionError> {
        // First download and add these top level dependencies crates to the Driver
        for (dep_pkg_name, pkg_src) in manifest.dependencies.iter() {
            let (dir_path, dep_meta) = Resolver::cache_dep(pkg_src)?;

            let (entry_path, crate_type) = (&dep_meta.entry_path, &dep_meta.crate_type);

            if crate_type == &CrateType::Binary {
                return Err(DependencyResolutionError::BinaryDependency {
                    dep_pkg_name: dep_pkg_name.to_string(),
                });
            }

            let crate_id = self.driver.create_non_local_crate(entry_path, *crate_type);
            self.driver.add_dep(parent_crate, crate_id, dep_pkg_name);

            self.cached_packages.insert(dir_path, (crate_id, dep_meta));
        }

        // Resolve all transitive dependencies
        for (dependency_path, (crate_id, dep_meta)) in self.cached_packages.iter() {
            if dep_meta.remote && manifest.has_local_path() {
                return Err(DependencyResolutionError::RemoteDepWithLocalDep {
                    dependency_path: dependency_path.to_path_buf(),
                });
            }
            let mut new_res = Resolver::with_driver(self.driver);
            new_res.resolve_manifest(*crate_id, dep_meta.manifest.clone())?;
        }
        Ok(())
    }

    /// If the dependency is remote, download the dependency
    /// and return the directory path along with the metadata
    /// Needed to fill the CachedDep struct
    ///
    /// If it's a local path, the same applies, however it will not
    /// be downloaded
    fn cache_dep(dep: &Dependency) -> Result<(PathBuf, CachedDep), DependencyResolutionError> {
        fn retrieve_meta(
            dir_path: &Path,
            remote: bool,
        ) -> Result<CachedDep, DependencyResolutionError> {
            let (entry_path, crate_type) = super::lib_or_bin(dir_path)?;
            let manifest_path = super::find_package_manifest(dir_path)?;
            let manifest = super::manifest::parse(manifest_path)?;
            Ok(CachedDep { entry_path, crate_type, manifest, remote })
        }

        match dep {
            Dependency::Github { git, tag } => {
                let dir_path =
                    clone_git_repo(git, tag).map_err(DependencyResolutionError::GitError)?;
                let meta = retrieve_meta(&dir_path, true)?;
                Ok((dir_path, meta))
            }
            Dependency::Path { path } => {
                let dir_path = std::path::PathBuf::from(path);
                let meta = retrieve_meta(&dir_path, false)?;
                Ok((dir_path, meta))
            }
        }
    }
}

// This needs to be public to support the tests in `cli/mod.rs`.
pub(crate) fn add_std_lib(driver: &mut Driver) {
    let std_crate_name = "std";
    let path_to_std_lib_file = PathBuf::from(std_crate_name).join("lib.nr");
    let std_crate = driver.create_non_local_crate(path_to_std_lib_file, CrateType::Library);
    driver.propagate_dep(std_crate, &CrateName::new(std_crate_name).unwrap());
}
