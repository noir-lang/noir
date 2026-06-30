#![forbid(unsafe_code)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
    str::FromStr,
};

use errors::SemverError;
use fm::{FILE_EXTENSION, NormalizePath};
use nargo::{
    constants::PKG_FILE,
    package::{Dependency, Package, PackageType},
    workspace::Workspace,
};
use noirc_frontend::{elaborator::UnstableFeature, graph::CrateName};
use serde::Deserialize;

mod deserializers;
mod errors;
mod flock;
mod git;
mod semver;

pub use errors::ManifestError;
pub use git::list_cached_git_dependencies;
use git::{clone_git_repo, lock_git_deps};

/// Searches for a `Nargo.toml` file in the current directory and all parent directories.
/// For example, if the current directory is `/workspace/package/src`, then this function
/// will search for a `Nargo.toml` file in
/// * `/workspace/package/src`,
/// * `/workspace/package`,
/// * `/workspace`.
///
/// Returns the [`PathBuf`] of the `Nargo.toml` file if found, otherwise returns None.
///
/// It will return innermost `Nargo.toml` file, which is the one closest to the current directory.
/// For example, if the current directory is `/workspace/package/src`, then this function
/// will return the `Nargo.toml` file in `/workspace/package/Nargo.toml`
pub fn find_file_manifest(current_path: &Path) -> Option<PathBuf> {
    for path in current_path.ancestors() {
        if let Ok(toml_path) = get_package_manifest(path) {
            return Some(toml_path);
        }
    }
    None
}

/// Returns the [`PathBuf`] of the directory containing the `Nargo.toml` by searching from `current_path` to the root of its [Path].
/// When `workspace` is `true` it returns the topmost directory, when `false` the innermost one.
///
/// Returns a [`ManifestError`] if no parent directories of `current_path` contain a manifest file.
pub fn find_root(current_path: &Path, workspace: bool) -> Result<PathBuf, ManifestError> {
    if workspace { find_package_root(current_path) } else { find_file_root(current_path) }
}

/// Returns the [`PathBuf`] of the directory containing the `Nargo.toml` by searching from `current_path` to the root of its [Path],
/// returning at the innermost directory found, i.e. the one corresponding to the package that contains the `current_path`.
///
/// Returns a [`ManifestError`] if no parent directories of `current_path` contain a manifest file.
pub fn find_file_root(current_path: &Path) -> Result<PathBuf, ManifestError> {
    match find_file_manifest(current_path) {
        Some(manifest_path) => {
            let package_root = manifest_path
                .parent()
                .expect("infallible: manifest file path can't be root directory");
            Ok(package_root.to_path_buf())
        }
        None => Err(ManifestError::MissingFile(current_path.to_path_buf())),
    }
}

/// Returns the [`PathBuf`] of the directory containing the `Nargo.toml` by searching from `current_path` to the root of its [Path],
/// returning the topmost directory found, i.e. the one corresponding to the entire workspace.
///
/// Returns a [`ManifestError`] if none of the ancestor directories of `current_path` contain a manifest file.
pub fn find_package_root(current_path: &Path) -> Result<PathBuf, ManifestError> {
    let root = path_root(current_path);
    let manifest_path = find_package_manifest(&root, current_path)?;

    let package_root =
        manifest_path.parent().expect("infallible: manifest file path can't be root directory");

    Ok(package_root.to_path_buf())
}

// TODO(#2323): We are probably going to need a "filepath utils" crate soon
/// Get the root of path, for example:
/// * `C:\foo\bar` -> `C:\foo`
/// * `//shared/foo/bar` -> `//shared/foo`
/// * `/foo` -> `/foo`
///   otherwise empty path.
fn path_root(path: &Path) -> PathBuf {
    let mut components = path.components();

    match (components.next(), components.next()) {
        // Preserve prefix if one exists
        (Some(prefix @ Component::Prefix(_)), Some(root @ Component::RootDir)) => {
            PathBuf::from(prefix.as_os_str()).join(root.as_os_str())
        }
        (Some(root @ Component::RootDir), _) => PathBuf::from(root.as_os_str()),
        _ => PathBuf::new(),
    }
}

/// Returns the [`PathBuf`] of the `Nargo.toml` file by searching from `current_path` and stopping at `root_path`.
///
/// Returns a [`ManifestError`] if no parent directories of `current_path` contain a manifest file.
pub fn find_package_manifest(
    root_path: &Path,
    current_path: &Path,
) -> Result<PathBuf, ManifestError> {
    if current_path.starts_with(root_path) {
        let mut found_toml_paths = Vec::new();
        for path in current_path.ancestors() {
            if let Ok(toml_path) = get_package_manifest(path) {
                found_toml_paths.push(toml_path);
            }
            // While traversing, break once we process the root specified
            if path == root_path {
                break;
            }
        }

        // Return the shallowest Nargo.toml, which will be the last in the list
        found_toml_paths.pop().ok_or_else(|| ManifestError::MissingFile(current_path.to_path_buf()))
    } else {
        Err(ManifestError::NoCommonAncestor {
            root: root_path.to_path_buf(),
            current: current_path.to_path_buf(),
        })
    }
}

/// Returns the [`PathBuf`] of the `Nargo.toml` file in the `current_path` directory.
///
/// Returns a [`ManifestError`] if `current_path` does not contain a manifest file.
pub fn get_package_manifest(current_path: &Path) -> Result<PathBuf, ManifestError> {
    let toml_path = current_path.join(PKG_FILE);
    if toml_path.exists() {
        Ok(toml_path)
    } else {
        Err(ManifestError::MissingFile(current_path.to_path_buf()))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct PackageConfig {
    pub package: PackageMetadata,
    #[serde(default)]
    pub dependencies: BTreeMap<String, DependencyConfig>,
}

impl PackageConfig {
    fn resolve_to_package(
        &self,
        root_dir: &Path,
        processed: &mut Vec<String>,
        assume_default_entry: bool, // assume that the 'default_entry_path' exists, e.g. src/main.nr
    ) -> Result<Package, ManifestError> {
        let name = &self.package.name;
        let name: CrateName = name.parse().map_err(|_| ManifestError::InvalidPackageName {
            toml: root_dir.join(PKG_FILE),
            name: name.into(),
        })?;

        let mut dependencies: BTreeMap<CrateName, Dependency> = BTreeMap::new();
        for (name, dep_config) in &self.dependencies {
            let name = name.parse().map_err(|_| ManifestError::InvalidDependencyName {
                toml: root_dir.join(PKG_FILE),
                name: name.into(),
            })?;
            let resolved_dep = dep_config.resolve_to_dependency(root_dir, processed)?;

            dependencies.insert(name, resolved_dep);
        }

        let package_type = match self.package.package_type.as_deref() {
            Some("lib") => PackageType::Library,
            Some("bin") => PackageType::Binary,
            Some("contract") => PackageType::Contract,
            Some(invalid) => {
                return Err(ManifestError::InvalidPackageType(
                    root_dir.join(PKG_FILE),
                    invalid.to_string(),
                ));
            }
            None => return Err(ManifestError::MissingPackageType(root_dir.join(PKG_FILE))),
        };

        let entry_path = if let Some(entry_path) = &self.package.entry {
            let custom_entry_path = root_dir.join(entry_path);
            if custom_entry_path.exists() {
                custom_entry_path
            } else {
                return Err(ManifestError::MissingEntryFile {
                    toml: root_dir.join(PKG_FILE),
                    entry: custom_entry_path,
                });
            }
        } else {
            let default_entry_path = match package_type {
                PackageType::Library => {
                    root_dir.join("src").join("lib").with_extension(FILE_EXTENSION)
                }
                PackageType::Binary | PackageType::Contract => {
                    root_dir.join("src").join("main").with_extension(FILE_EXTENSION)
                }
            };

            if default_entry_path.exists() || assume_default_entry {
                default_entry_path
            } else {
                return Err(ManifestError::MissingDefaultEntryFile {
                    toml: root_dir.join(PKG_FILE),
                    entry: default_entry_path,
                    package_type,
                });
            }
        };

        // If there is a package version, ensure that it is semver compatible
        if let Some(version) = &self.package.version {
            semver::parse_semver_compatible_version(version).map_err(|err| {
                ManifestError::SemverError(SemverError::CouldNotParsePackageVersion {
                    package_name: name.to_string(),
                    error: err.to_string(),
                })
            })?;
        }

        // Collect any unstable features the package needs to compile.
        // Ignore the ones that we don't recognize: maybe they are no longer unstable, but a dependency hasn't been updated.
        let compiler_required_unstable_features =
            self.package.compiler_unstable_features.as_ref().map_or(Vec::new(), |feats| {
                feats.iter().filter_map(|feat| UnstableFeature::from_str(feat).ok()).collect()
            });

        Ok(Package {
            version: self.package.version.clone(),
            compiler_required_version: self.package.compiler_version.clone(),
            compiler_required_unstable_features,
            root_dir: root_dir.to_path_buf(),
            entry_path,
            package_type,
            name,
            dependencies,
        })
    }
}

/// Contains all the information about a package, as loaded from a `Nargo.toml`.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Config {
    /// Represents a `Nargo.toml` with package fields.
    Package { package_config: PackageConfig },
    /// Represents a `Nargo.toml` with workspace fields.
    Workspace { workspace_config: WorkspaceConfig },
}

impl TryFrom<String> for Config {
    type Error = toml::de::Error;

    fn try_from(toml: String) -> Result<Self, Self::Error> {
        toml::from_str(&toml)
    }
}

impl TryFrom<&str> for Config {
    type Error = toml::de::Error;

    fn try_from(toml: &str) -> Result<Self, Self::Error> {
        toml::from_str(toml)
    }
}

/// Tracks the `root_dir` of a `Nargo.toml` and the contents inside the file.
pub struct NargoToml {
    pub root_dir: PathBuf,
    pub config: Config,
}

#[derive(Default, Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct WorkspaceConfig {
    /// List of members in this workspace.
    pub members: Vec<PathBuf>,
    /// Specifies the default crate to interact with in the context (similarly to how we have nargo as the default crate in this repository).
    pub default_member: Option<PathBuf>,
}

#[allow(dead_code)]
#[derive(Default, Debug, Deserialize, Clone)]
pub struct PackageMetadata {
    pub name: String,
    pub version: Option<String>,
    #[serde(alias = "type")]
    pub package_type: Option<String>,
    pub entry: Option<PathBuf>,
    pub description: Option<String>,
    pub authors: Option<Vec<String>>,
    // If no compiler version is supplied, the latest is used
    // For now, we state that all packages must be compiled under the same
    // compiler version.
    // We also state that ACIR and the compiler will upgrade in lockstep.
    // so you will not need to supply an ACIR and compiler version
    pub compiler_version: Option<String>,
    /// List of unstable features we want the compiler to enable to compile this package.
    /// This is most useful with the LSP, so it can figure out what is allowed without CLI args.
    pub compiler_unstable_features: Option<Vec<String>>,
    pub license: Option<String>,
    pub expression_width: Option<String>,
}

#[derive(Debug, Clone)]
/// Enum representing the different types of ways to
/// supply a source for the dependency
pub enum DependencyConfig {
    Git { git: String, tag: String, directory: Option<String> },
    Path { path: String },
}

impl DependencyConfig {
    fn resolve_to_dependency(
        &self,
        pkg_root: &Path,
        processed: &mut Vec<String>,
    ) -> Result<Dependency, ManifestError> {
        let dep = match self {
            Self::Git { git, tag, directory } => {
                let dir_path = clone_git_repo(git, tag).map_err(ManifestError::GitError)?;
                let project_path = if let Some(directory) = directory {
                    let internal_path = dir_path.join(directory).normalize();
                    if !internal_path.starts_with(&dir_path) {
                        return Err(ManifestError::InvalidDirectory {
                            toml: pkg_root.join(PKG_FILE),
                            directory: directory.into(),
                        });
                    }
                    internal_path
                } else {
                    dir_path
                };
                let toml_path = project_path.join(PKG_FILE);
                let package = resolve_package_from_toml(&toml_path, processed)?;
                Dependency::Remote { package }
            }
            Self::Path { path } => {
                let dir_path = pkg_root.join(path);
                let toml_path = dir_path.join(PKG_FILE);
                let package = resolve_package_from_toml(&toml_path, processed)?;
                Dependency::Local { package }
            }
        };

        // Cannot depend on a binary
        // TODO: Can we depend upon contracts?
        if dep.is_binary() {
            Err(ManifestError::BinaryDependency(dep.package_name().clone()))
        } else {
            Ok(dep)
        }
    }
}

/// Resolves a single dependency relative to `pkg_root`, downloading it into the global cache
/// if it is a git dependency that hasn't been fetched yet.
///
/// This performs the same validation as workspace resolution for one entry: it checks that the
/// dependency's `Nargo.toml` exists and that it is not a binary package. The returned
/// [`Dependency`] carries the on-disk [`Package`], whose `root_dir` is where the source lives.
pub fn resolve_dependency(
    pkg_root: &Path,
    dep: &DependencyConfig,
) -> Result<Dependency, ManifestError> {
    let _lock = lock_git_deps().expect("Failed to lock git dependencies cache");
    dep.resolve_to_dependency(pkg_root, &mut Vec::new())
}

/// Adds a dependency named `name` to the `[dependencies]` table of the manifest at `toml_path`,
/// preserving the file's existing formatting and comments.
///
/// If a dependency with the same name is already present and `overwrite` is `false`, this returns
/// [`ManifestError::DependencyAlreadyExists`] and leaves the file untouched.
pub fn add_dependency_to_manifest(
    toml_path: &Path,
    name: &str,
    dep: &DependencyConfig,
    overwrite: bool,
) -> Result<(), ManifestError> {
    use toml_edit::{DocumentMut, Item, Table};

    let contents = std::fs::read_to_string(toml_path)
        .map_err(|_| ManifestError::ReadFailed(toml_path.to_path_buf()))?;
    let mut doc = contents.parse::<DocumentMut>()?;

    let dependencies = doc
        .entry("dependencies")
        .or_insert_with(|| Item::Table(Table::new()))
        .as_table_mut()
        .ok_or_else(|| ManifestError::InvalidDependenciesTable(toml_path.to_path_buf()))?;

    if dependencies.contains_key(name) && !overwrite {
        return Err(ManifestError::DependencyAlreadyExists(name.to_string()));
    }

    let mut entry = toml_edit::InlineTable::new();
    match dep {
        DependencyConfig::Path { path } => {
            entry.insert("path", path.as_str().into());
        }
        DependencyConfig::Git { git, tag, directory } => {
            entry.insert("git", git.as_str().into());
            entry.insert("tag", tag.as_str().into());
            if let Some(directory) = directory {
                entry.insert("directory", directory.as_str().into());
            }
        }
    }
    dependencies[name] = toml_edit::value(entry);

    std::fs::write(toml_path, doc.to_string())
        .map_err(|_| ManifestError::WriteFailed(toml_path.to_path_buf()))?;

    Ok(())
}

fn toml_to_workspace(
    nargo_toml: NargoToml,
    package_selection: PackageSelection,
    assume_default_entry: bool, // assume that the 'default_entry_path' exists, e.g. src/main.nr
) -> Result<Workspace, ManifestError> {
    let mut resolved = Vec::new();
    let _lock = lock_git_deps().expect("Failed to lock git dependencies cache");
    let workspace = match nargo_toml.config {
        Config::Package { package_config } => {
            let member = package_config.resolve_to_package(
                &nargo_toml.root_dir,
                &mut resolved,
                assume_default_entry,
            )?;
            match &package_selection {
                PackageSelection::Selected(selected_name) if selected_name != &member.name => {
                    return Err(ManifestError::MissingSelectedPackage(member.name));
                }
                _ => Workspace {
                    root_dir: nargo_toml.root_dir,
                    selected_package_index: Some(0),
                    members: vec![member],
                    is_assumed: false,
                    target_dir: None,
                },
            }
        }
        Config::Workspace { workspace_config } => {
            let mut members = Vec::new();
            let mut selected_package_index = None;
            for (index, member_path) in workspace_config.members.into_iter().enumerate() {
                let package_root_dir = nargo_toml.root_dir.join(&member_path);
                let package_toml_path = package_root_dir.join(PKG_FILE);
                let member = resolve_package_from_toml(&package_toml_path, &mut resolved)?;

                match &package_selection {
                    PackageSelection::Selected(selected_name) => {
                        if &member.name == selected_name {
                            selected_package_index = Some(index);
                        }
                    }
                    PackageSelection::DefaultOrAll => {
                        if Some(&member_path) == workspace_config.default_member.as_ref() {
                            selected_package_index = Some(index);
                        }
                    }
                    PackageSelection::All => selected_package_index = None,
                }

                members.push(member);
            }

            // If the selected_package_index is still `None` but we have see a default_member or selected package,
            // we want to present an error to users
            match package_selection {
                PackageSelection::Selected(selected_name) => {
                    if selected_package_index.is_none() {
                        return Err(ManifestError::MissingSelectedPackage(selected_name));
                    }
                }
                PackageSelection::DefaultOrAll => match workspace_config.default_member {
                    // If `default-member` is specified but we don't have a selected_package_index, we need to fail
                    Some(default_path) if selected_package_index.is_none() => {
                        return Err(ManifestError::MissingDefaultPackage(default_path));
                    }
                    // However, if there wasn't a `default-member`, we select All, so no error is needed
                    _ => (),
                },
                PackageSelection::All => (),
            }

            Workspace {
                root_dir: nargo_toml.root_dir,
                members,
                selected_package_index,
                is_assumed: false,
                target_dir: None,
            }
        }
    };

    Ok(workspace)
}

/// Attempts to read the file at the provided `toml_path` as a `Nargo.toml`
/// file, returning it if the read was successful.
///
/// # Errors
///
/// - [`ManifestError::ReadFailed`] if the file could not be read.
/// - [`ManifestError::MissingParent`] if the Nargo.toml file did not have a
///   parent directory.
/// - [`toml::de::Error`] if the Nargo.toml file could not be converted to a
///   valid [`Config`].
pub fn read_toml(toml_path: &Path) -> Result<NargoToml, ManifestError> {
    let toml_path = toml_path.normalize();
    let toml_as_string = std::fs::read_to_string(&toml_path)
        .map_err(|_| ManifestError::ReadFailed(toml_path.clone()))?;
    let root_dir = toml_path.parent().ok_or(ManifestError::MissingParent)?;
    let nargo_toml =
        NargoToml { root_dir: root_dir.to_path_buf(), config: toml_as_string.try_into()? };

    Ok(nargo_toml)
}

/// Resolves a Nargo.toml file into a `Package` struct as defined by our `nargo` core.
fn resolve_package_from_toml(
    toml_path: &Path,
    processed: &mut Vec<String>,
) -> Result<Package, ManifestError> {
    // Normalize the path so a manifest reached through different spellings of
    // the same location (e.g. `a/../b` and `b`) maps to a single entry. This
    // keeps cycle detection consistent with `read_toml`, which derives each
    // package's root directory from the normalized path.
    let toml_path = toml_path.normalize();
    // Checks for cyclic dependencies
    let str_path = toml_path.to_str().expect("ICE - path is empty");
    if processed.contains(&str_path.to_string()) {
        let mut cycle = false;
        let mut message = String::new();
        for toml in processed {
            cycle = cycle || toml == str_path;
            if cycle {
                message += &format!("{toml} referencing ");
            }
        }
        message += str_path;
        return Err(ManifestError::CyclicDependency { cycle: message });
    }
    // Adds the package to the set of resolved packages
    if let Some(str) = toml_path.to_str() {
        processed.push(str.to_string());
    }

    let nargo_toml = read_toml(&toml_path)?;

    let result = match nargo_toml.config {
        Config::Package { package_config } => {
            let assume_default_entry = false;
            package_config.resolve_to_package(&nargo_toml.root_dir, processed, assume_default_entry)
        }
        Config::Workspace { .. } => Err(ManifestError::UnexpectedWorkspace(toml_path.clone())),
    };
    let pos =
        processed.iter().position(|toml| toml == str_path).expect("added package must be here");
    processed.remove(pos);
    result
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PackageSelection {
    Selected(CrateName),
    DefaultOrAll,
    All,
}

/// Resolves a Nargo.toml file into a `Workspace` struct as defined by our `nargo` core.
///
/// As a side effect it downloads project dependencies as well.
pub fn resolve_workspace_from_toml(
    toml_path: &Path,
    package_selection: PackageSelection,
    current_compiler_version: Option<String>,
) -> Result<Workspace, ManifestError> {
    let nargo_toml = read_toml(toml_path)?;
    let assume_default_entry = false;
    resolve_workspace_from_fixed_toml(
        nargo_toml,
        package_selection,
        current_compiler_version,
        assume_default_entry,
    )
}

/// Resolves a Nargo.toml _ into a `Workspace` struct as defined by our `nargo` core.
///
/// As a side effect it downloads project dependencies as well.
pub fn resolve_workspace_from_fixed_toml(
    nargo_toml: NargoToml,
    package_selection: PackageSelection,
    current_compiler_version: Option<String>,
    assume_default_entry: bool,
) -> Result<Workspace, ManifestError> {
    let workspace = toml_to_workspace(nargo_toml, package_selection, assume_default_entry)?;
    if let Some(current_compiler_version) = current_compiler_version {
        semver::semver_check_workspace(&workspace, current_compiler_version)?;
    }
    Ok(workspace)
}

#[cfg(test)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        str::FromStr,
    };

    use test_case::test_matrix;

    use nargo::constants::PKG_FILE;

    use crate::{Config, DependencyConfig, ManifestError, add_dependency_to_manifest, find_root};

    /// Writes `contents` to a `Nargo.toml` in a fresh temp dir and returns the temp dir and the
    /// manifest path. The temp dir must be kept alive for the duration of the test.
    fn manifest_with(contents: &str) -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let toml_path = tmp.path().join(PKG_FILE);
        std::fs::write(&toml_path, contents).unwrap();
        (tmp, toml_path)
    }

    #[test]
    fn add_path_dependency_preserves_existing_content() {
        let (_tmp, toml_path) = manifest_with(
            r#"[package]
name = "demo"
type = "bin"
authors = [""]

[dependencies]
# keep me around
existing = { path = "../existing" }
"#,
        );

        add_dependency_to_manifest(
            &toml_path,
            "my_lib",
            &DependencyConfig::Path { path: "../my_lib".to_string() },
            false,
        )
        .unwrap();

        insta::assert_snapshot!(std::fs::read_to_string(&toml_path).unwrap(), @r#"
        [package]
        name = "demo"
        type = "bin"
        authors = [""]

        [dependencies]
        # keep me around
        existing = { path = "../existing" }
        my_lib = { path = "../my_lib" }
        "#);
    }

    #[test]
    fn add_git_dependency_with_directory() {
        let (_tmp, toml_path) = manifest_with(
            r#"[package]
name = "demo"
type = "bin"
authors = [""]

[dependencies]
"#,
        );

        add_dependency_to_manifest(
            &toml_path,
            "bignum",
            &DependencyConfig::Git {
                git: "https://github.com/noir-lang/noir-bignum".to_string(),
                tag: "v0.4.2".to_string(),
                directory: Some("crates/bignum".to_string()),
            },
            false,
        )
        .unwrap();

        insta::assert_snapshot!(std::fs::read_to_string(&toml_path).unwrap(), @r#"
        [package]
        name = "demo"
        type = "bin"
        authors = [""]

        [dependencies]
        bignum = { git = "https://github.com/noir-lang/noir-bignum", tag = "v0.4.2", directory = "crates/bignum" }
        "#);
    }

    #[test]
    fn add_dependency_creates_dependencies_section_when_absent() {
        let (_tmp, toml_path) = manifest_with(
            r#"[package]
name = "demo"
type = "bin"
authors = [""]
"#,
        );

        add_dependency_to_manifest(
            &toml_path,
            "my_lib",
            &DependencyConfig::Path { path: "../my_lib".to_string() },
            false,
        )
        .unwrap();

        insta::assert_snapshot!(std::fs::read_to_string(&toml_path).unwrap(), @r#"
        [package]
        name = "demo"
        type = "bin"
        authors = [""]

        [dependencies]
        my_lib = { path = "../my_lib" }
        "#);
    }

    #[test]
    fn add_existing_dependency_without_override_errors_and_leaves_file_unchanged() {
        let original = r#"[package]
name = "demo"
type = "bin"
authors = [""]

[dependencies]
my_lib = { path = "../my_lib" }
"#;
        let (_tmp, toml_path) = manifest_with(original);

        let error = add_dependency_to_manifest(
            &toml_path,
            "my_lib",
            &DependencyConfig::Path { path: "../somewhere_else".to_string() },
            false,
        )
        .expect_err("adding an existing dependency without --override should fail");

        assert!(matches!(error, ManifestError::DependencyAlreadyExists(name) if name == "my_lib"));
        assert_eq!(
            std::fs::read_to_string(&toml_path).unwrap(),
            original,
            "the manifest must be left byte-for-byte unchanged"
        );
    }

    #[test]
    fn add_existing_dependency_with_override_replaces_entry() {
        let (_tmp, toml_path) = manifest_with(
            r#"[package]
name = "demo"
type = "bin"
authors = [""]

[dependencies]
my_lib = { path = "../my_lib" }
"#,
        );

        add_dependency_to_manifest(
            &toml_path,
            "my_lib",
            &DependencyConfig::Path { path: "../somewhere_else".to_string() },
            true,
        )
        .unwrap();

        insta::assert_snapshot!(std::fs::read_to_string(&toml_path).unwrap(), @r#"
        [package]
        name = "demo"
        type = "bin"
        authors = [""]

        [dependencies]
        my_lib = { path = "../somewhere_else" }
        "#);
    }

    #[test]
    fn resolve_path_dependency_yields_package_name_and_rejects_binaries() {
        use crate::resolve_dependency;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // Create a package of the given `type` whose declared name differs from its directory,
        // so the test exercises name discovery rather than assuming the two coincide.
        let write_pkg = |dir: &str, name: &str, package_type: &str| {
            let pkg_dir = root.join(dir);
            std::fs::create_dir_all(pkg_dir.join("src")).unwrap();
            std::fs::write(
                pkg_dir.join(PKG_FILE),
                format!(
                    "[package]\nname = \"{name}\"\ntype = \"{package_type}\"\nauthors = [\"\"]\n"
                ),
            )
            .unwrap();
            let entry = if package_type == "lib" { "lib.nr" } else { "main.nr" };
            std::fs::write(pkg_dir.join("src").join(entry), "").unwrap();
        };

        write_pkg("the_lib", "cool_lib", "lib");
        write_pkg("the_bin", "cool_bin", "bin");

        let dep = resolve_dependency(root, &DependencyConfig::Path { path: "the_lib".to_string() })
            .expect("a library path dependency should resolve");
        assert_eq!(dep.package_name().to_string(), "cool_lib");

        match resolve_dependency(root, &DependencyConfig::Path { path: "the_bin".to_string() }) {
            Err(ManifestError::BinaryDependency(name)) => assert_eq!(name.to_string(), "cool_bin"),
            _ => panic!("a binary path dependency should be rejected"),
        }
    }

    #[test]
    fn parse_standard_toml() {
        let src = r#"

        [package]
        name = "test"
        authors = ["kev", "foo"]
        compiler_version = "*"

        [dependencies]
        rand = { tag = "next", git = "https://github.com/rust-lang-nursery/rand"}
        cool = { tag = "next", git = "https://github.com/rust-lang-nursery/rand"}
        hello = {path = "./noir_driver"}
    "#;

        assert!(Config::try_from(String::from(src)).is_ok());
        assert!(Config::try_from(src).is_ok());
    }

    #[test]
    fn parse_package_toml_no_deps() {
        let src = r#"
        [package]
        name = "test"
        authors = ["kev", "foo"]
        compiler_version = "*"
    "#;

        assert!(Config::try_from(String::from(src)).is_ok());
        assert!(Config::try_from(src).is_ok());
    }

    #[test]
    fn parse_workspace_toml() {
        let src = r#"
        [workspace]
        members = ["a", "b"]
    "#;

        assert!(Config::try_from(String::from(src)).is_ok());
        assert!(Config::try_from(src).is_ok());
    }

    #[test]
    fn parse_workspace_default_member_toml() {
        let src = r#"
        [workspace]
        members = ["a", "b"]
        default-member = "a"
    "#;

        assert!(Config::try_from(String::from(src)).is_ok());
        assert!(Config::try_from(src).is_ok());
    }

    #[test]
    fn parse_package_expression_width_toml() {
        let src = r#"
    [package]
    name = "test"
    version = "0.1.0"
    type = "bin"
    authors = [""]
    expression_width = "3"
    "#;

        assert!(Config::try_from(String::from(src)).is_ok());
        assert!(Config::try_from(src).is_ok());
    }

    /// Test that `find_root` handles all kinds of prefixes.
    /// (It dispatches based on `workspace` to methods which handle paths differently).
    #[test_matrix(
        [true, false],
        ["C:\\foo\\bar", "//shared/foo/bar", "/foo/bar", "bar/baz", ""]
    )]
    fn test_find_root_does_not_panic(workspace: bool, path: &str) {
        let path = PathBuf::from_str(path).unwrap();
        let error = find_root(&path, workspace).expect_err("non-existing paths");
        assert!(matches!(error, ManifestError::MissingFile(_)));
    }

    /// Test to demonstrate how `find_root` works.
    #[test]
    fn test_find_root_example() {
        const INDENT_SIZE: usize = 4;
        /// Create directories and files according to a YAML-like layout below
        fn setup(layout: &str, root: &Path) {
            fn is_dir(item: &str) -> bool {
                !item.contains('.')
            }
            let mut current_dir = root.to_path_buf();
            let mut current_indent = 0;
            let mut last_item: Option<String> = None;

            for line in layout.lines() {
                if let Some((prefix, item)) = line.split_once('-') {
                    let item = item.replace(std::path::MAIN_SEPARATOR, "_").trim().to_string();

                    let indent = prefix.len() / INDENT_SIZE;

                    if last_item.is_none() {
                        current_indent = indent;
                    }

                    assert!(
                        indent <= current_indent + 1,
                        "cannot increase indent by more than {INDENT_SIZE}; item = {item}, current_dir={}",
                        current_dir.display()
                    );

                    // Go into the last created directory
                    if let Some(last_item) = last_item
                        && indent > current_indent
                    {
                        assert!(is_dir(&last_item), "last item was not a dir: {last_item}");
                        current_dir.push(last_item);
                        current_indent += 1;
                    }
                    // Go back into an ancestor directory
                    while indent < current_indent {
                        current_dir.pop();
                        current_indent -= 1;
                    }
                    // Create a file or a directory
                    let item_path = current_dir.join(&item);
                    if is_dir(&item) {
                        std::fs::create_dir(&item_path).unwrap_or_else(|e| {
                            panic!("failed to create dir {}: {e}", item_path.display())
                        });
                    } else {
                        std::fs::write(&item_path, "").expect("failed to create file");
                    }

                    last_item = Some(item);
                }
            }
        }

        // Temporary directory to hold the project.
        let tmp = tempfile::tempdir().unwrap();
        // Join a string path to the tmp dir
        let path = |p: &str| tmp.path().join(p);
        // Check that an expected root is found
        let assert_ok = |current_dir: &str, ws: bool, exp: &str| {
            let root = find_root(&path(current_dir), ws).expect("should find a root");
            assert_eq!(root, path(exp));
        };
        // Check that a root is not found
        let assert_err = |current_dir: &str| {
            find_root(&path(current_dir), true).expect_err("shouldn't find a root");
        };

        let layout = r"
            - project
                - docs
                - workspace
                    - packages
                        - foo
                            - Nargo.toml
                            - Prover.toml
                            - src
                                - main.nr
                        - bar
                            - Nargo.toml
                            - src
                                - lib.nr
                    - Nargo.toml
                - examples
                    - baz
                        - Nargo.toml
                        - src
                            - main.nr
            ";

        // Set up the file system.
        setup(layout, tmp.path());

        assert_err("dummy");
        assert_err("project/docs");
        assert_err("project/examples");
        assert_ok("project/workspace", true, "project/workspace");
        assert_ok("project/workspace", false, "project/workspace");
        assert_ok("project/workspace/packages/foo", true, "project/workspace");
        assert_ok("project/workspace/packages/bar", false, "project/workspace/packages/bar");
        assert_ok("project/examples/baz/src", true, "project/examples/baz");
        assert_ok("project/examples/baz/src", false, "project/examples/baz");
    }

    /// A dependency cycle should be reported against normalized paths, no matter
    /// how the cyclic `path` dependencies are spelled in each `Nargo.toml`.
    #[test]
    fn cyclic_dependency_is_reported_with_normalized_paths() {
        use crate::{PackageSelection, resolve_workspace_from_toml};

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let write_lib = |name: &str, dep_name: &str, dep_path: &str| {
            let dir = root.join(name);
            std::fs::create_dir_all(dir.join("src")).unwrap();
            std::fs::write(
                dir.join(PKG_FILE),
                format!(
                    "[package]\nname = \"{name}\"\ntype = \"lib\"\nauthors = [\"\"]\n\n[dependencies]\n{dep_name} = {{ path = \"{dep_path}\" }}\n"
                ),
            )
            .unwrap();
            std::fs::write(dir.join("src").join("lib.nr"), "").unwrap();
        };

        // `a` depends on `b` and `b` depends on `a`, forming a cycle. The `path`
        // dependencies are spelled with `..` segments so the raw and normalized
        // forms of each manifest path differ.
        write_lib("a", "b", "../b");
        write_lib("b", "a", "../a");

        let error =
            resolve_workspace_from_toml(&root.join("a/Nargo.toml"), PackageSelection::All, None)
                .err()
                .expect("a <-> b is a dependency cycle");

        let ManifestError::CyclicDependency { cycle } = error else {
            panic!("expected a cyclic dependency error, got: {error:?}");
        };

        assert!(
            !cycle.contains(".."),
            "cycle should be reported with normalized paths, got: {cycle}"
        );
    }
}
