use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use nargo::manifest::{Dependency, Manifest, PackageManifest};
use noirc_driver::{add_dep, create_local_crate, create_non_local_crate};
use noirc_frontend::{
    graph::{CrateId, CrateType},
    hir::Context,
};
use thiserror::Error;

use crate::{git::clone_git_repo, InvalidPackageError};

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

    /// Workspace does not contain packages
    #[error("manifest path `{}` contains no packages", path.display())]
    EmptyWorkspace { path: PathBuf },

    /// Use workspace as a dependency is not currently supported
    #[error("use workspace as a dependency is not currently supported")]
    WorkspaceDependency,
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

/// Returns the Driver and the backend to use
/// Note that the backend is ignored in the dependencies.
/// Since Noir is backend agnostic, this is okay to do.
/// XXX: Need to handle when a local package changes!
pub(crate) fn resolve_root_manifest(
    dir_path: &std::path::Path,
) -> Result<Context, DependencyResolutionError> {
    let mut context = Context::default();

    let manifest_path = super::find_package_manifest(dir_path)?;
    let manifest = super::manifest::parse(&manifest_path)?;

    match manifest {
        Manifest::Package(package) => {
            let (entry_path, crate_type) = super::lib_or_bin(dir_path)?;
            let crate_id = create_local_crate(&mut context, entry_path, crate_type);

            let pkg_root = manifest_path.parent().expect("Every manifest path has a parent.");
            resolve_manifest(&mut context, crate_id, package, pkg_root)?;
        }
        Manifest::Workspace(workspace) => {
            let config = workspace.config;
            let members = config.members;

            let maybe_local = config
                .default_member
                .or_else(|| members.last().cloned())
                .map(|member| dir_path.join(member));

            let default_member = match maybe_local {
                Some(member) => member,
                None => {
                    return Err(DependencyResolutionError::EmptyWorkspace { path: manifest_path })
                }
            };

            let (entry_path, _crate_type) = super::lib_or_bin(default_member)?;
            let _local = create_local_crate(&mut context, entry_path, CrateType::Workspace);

            for member in members {
                let path: PathBuf = dir_path.join(member);
                let (entry_path, crate_type) = super::lib_or_bin(path)?;
                create_non_local_crate(&mut context, entry_path, crate_type);
            }
        }
    };

    Ok(context)
}

// Resolves a config file by recursively resolving the dependencies in the config
// Need to solve the case of a project trying to use itself as a dep
//
// We do not need to add stdlib, as it's implicitly
// imported. However, it may be helpful to have the stdlib imported by the
// package manager.
fn resolve_manifest(
    context: &mut Context,
    parent_crate: CrateId,
    manifest: PackageManifest,
    pkg_root: &Path,
) -> Result<(), DependencyResolutionError> {
    let mut cached_packages: HashMap<PathBuf, (CrateId, CachedDep)> = HashMap::new();

    // First download and add these top level dependencies crates to the Driver
    for (dep_pkg_name, pkg_src) in manifest.dependencies.iter() {
        let (dir_path, dep_meta) = cache_dep(pkg_src, pkg_root)?;

        let (entry_path, crate_type) = (&dep_meta.entry_path, &dep_meta.crate_type);

        if crate_type == &CrateType::Binary {
            return Err(DependencyResolutionError::BinaryDependency {
                dep_pkg_name: dep_pkg_name.to_string(),
            });
        }

        let crate_id = create_non_local_crate(context, entry_path, *crate_type);
        add_dep(context, parent_crate, crate_id, dep_pkg_name);

        cached_packages.insert(dir_path, (crate_id, dep_meta));
    }

    // Resolve all transitive dependencies
    for (dependency_path, (crate_id, dep_meta)) in cached_packages {
        if dep_meta.remote && dep_meta.manifest.has_local_dependency() {
            return Err(DependencyResolutionError::RemoteDepWithLocalDep { dependency_path });
        }
        // TODO: Why did it create a new resolver?
        resolve_manifest(context, crate_id, dep_meta.manifest, &dependency_path)?;
    }
    Ok(())
}

/// If the dependency is remote, download the dependency
/// and return the directory path along with the metadata
/// Needed to fill the CachedDep struct
///
/// If it's a local path, the same applies, however it will not
/// be downloaded
fn cache_dep(
    dep: &Dependency,
    pkg_root: &Path,
) -> Result<(PathBuf, CachedDep), DependencyResolutionError> {
    fn retrieve_meta(
        dir_path: &Path,
        remote: bool,
    ) -> Result<CachedDep, DependencyResolutionError> {
        let (entry_path, crate_type) = super::lib_or_bin(dir_path)?;
        let manifest_path = super::find_package_manifest(dir_path)?;
        let manifest = super::manifest::parse(manifest_path)?
            .to_package()
            .ok_or(DependencyResolutionError::WorkspaceDependency)?;
        Ok(CachedDep { entry_path, crate_type, manifest, remote })
    }

    match dep {
        Dependency::Github { git, tag } => {
            let dir_path = clone_git_repo(git, tag).map_err(DependencyResolutionError::GitError)?;
            let meta = retrieve_meta(&dir_path, true)?;
            Ok((dir_path, meta))
        }
        Dependency::Path { path } => {
            let dir_path = pkg_root.join(path);
            let meta = retrieve_meta(&dir_path, false)?;
            Ok((dir_path, meta))
        }
    }
}
