use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use nargo::manifest::{Dependency, Manifest, PackageManifest, WorkspaceConfig};
use noirc_driver::{add_dep, create_local_crate, create_non_local_crate};
use noirc_frontend::{
    graph::{CrateId, CrateName, CrateType},
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

    /// Multiple workspace roots found in the same workspace
    #[error("multiple workspace roots found in the same workspace:\n{}\n{}", root.display(), member.display())]
    MultipleWorkspace { root: PathBuf, member: PathBuf },

    /// Invalid character `-` in package name
    #[error("invalid character `-` in package name")]
    InvalidPackageName,

    #[error("package specification `{0}` did not match any packages")]
    PackageNotFound(String),

    #[error("two packages named `{0}` in this workspace")]
    PackageCollision(String),
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
    package: Option<String>,
) -> Result<(Context, CrateId), DependencyResolutionError> {
    let mut context = Context::default();

    let manifest_path = super::find_package_manifest(dir_path)?;
    let manifest = super::manifest::parse(&manifest_path)?;

    let crate_id = match manifest {
        Manifest::Package(package) => {
            let (entry_path, crate_type) = super::lib_or_bin(dir_path)?;

            let crate_id = create_local_crate(&mut context, entry_path, crate_type);
            let pkg_root = manifest_path.parent().expect("Every manifest path has a parent.");

            resolve_package_manifest(&mut context, crate_id, package, pkg_root)?;

            crate_id
        }
        Manifest::Workspace(workspace) => resolve_workspace_manifest(
            &mut context,
            package,
            manifest_path,
            dir_path,
            workspace.config,
        )?,
    };

    Ok((context, crate_id))
}

// Resolves a config file by recursively resolving the dependencies in the config
// Need to solve the case of a project trying to use itself as a dep
//
// We do not need to add stdlib, as it's implicitly
// imported. However, it may be helpful to have the stdlib imported by the
// package manager.
fn resolve_package_manifest(
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
        resolve_package_manifest(context, crate_id, dep_meta.manifest, &dependency_path)?;
    }
    Ok(())
}

fn crate_name(name: Option<CrateName>) -> String {
    name.map(|name| name.as_string()).unwrap_or_else(|| "[unnamed]".to_string())
}

fn resolve_workspace_manifest(
    context: &mut Context,
    mut local_package: Option<String>,
    manifest_path: PathBuf,
    dir_path: &Path,
    workspace: WorkspaceConfig,
) -> Result<CrateId, DependencyResolutionError> {
    let members = workspace.members;
    let mut packages = HashMap::new();

    if members.is_empty() {
        return Err(DependencyResolutionError::EmptyWorkspace { path: manifest_path });
    }

    for member in &members {
        let member_path: PathBuf = dir_path.join(member);
        let member_member_path = super::find_package_manifest(&member_path)?;
        let member_manifest = super::manifest::parse(&member_member_path)?;

        match member_manifest {
            Manifest::Package(inner) => {
                let name = inner
                    .package
                    .name
                    .map(|name| {
                        CrateName::new(&name)
                            .map_err(|_name| DependencyResolutionError::InvalidPackageName)
                    })
                    .transpose()?;

                if packages.insert(name.clone(), member_path).is_some() {
                    return Err(DependencyResolutionError::PackageCollision(crate_name(name)));
                }

                if local_package.is_none() && workspace.default_member.as_ref() == Some(member) {
                    local_package = name.as_ref().map(CrateName::as_string);
                }
            }
            Manifest::Workspace(_) => {
                return Err(DependencyResolutionError::MultipleWorkspace {
                    root: manifest_path,
                    member: member_member_path,
                })
            }
        }
    }

    let local_package = match local_package {
        Some(local_package) => CrateName::new(&local_package)
            .map_err(|_| DependencyResolutionError::InvalidPackageName)?
            .into(),
        None => packages.keys().last().expect("non-empty packages").clone(),
    };

    let local_crate = packages
        .remove(&local_package)
        .ok_or_else(|| DependencyResolutionError::PackageNotFound(crate_name(local_package)))?;

    let (entry_path, _crate_type) = super::lib_or_bin(local_crate)?;
    let crate_id = create_local_crate(context, entry_path, CrateType::Workspace);

    for (_, package_path) in packages.drain() {
        let (entry_path, crate_type) = super::lib_or_bin(package_path)?;
        create_non_local_crate(context, entry_path, crate_type);
    }

    Ok(crate_id)
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
