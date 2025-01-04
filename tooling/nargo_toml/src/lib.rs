#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

use errors::SemverError;
use fm::{NormalizePath, FILE_EXTENSION};
use nargo::{
    package::{Dependency, Package, PackageType},
    workspace::Workspace,
};
use noirc_driver::parse_expression_width;
use noirc_frontend::graph::CrateName;
use serde::Deserialize;

mod errors;
mod git;
mod semver;

pub use errors::ManifestError;
use git::clone_git_repo;

/// Searches for a `Nargo.toml` file in the current directory and all parent directories.
/// For example, if the current directory is `/workspace/package/src`, then this function
/// will search for a `Nargo.toml` file in
/// * `/workspace/package/src`,
/// * `/workspace/package`,
/// * `/workspace`.
///
/// Returns the [PathBuf] of the `Nargo.toml` file if found, otherwise returns None.
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

/// Returns the [PathBuf] of the directory containing the `Nargo.toml` by searching from `current_path` to the root of its [Path].
/// When `workspace` is `true` it returns the topmost directory, when `false` the innermost one.
///
/// Returns a [ManifestError] if no parent directories of `current_path` contain a manifest file.
pub fn find_root(current_path: &Path, workspace: bool) -> Result<PathBuf, ManifestError> {
    if workspace {
        find_package_root(current_path)
    } else {
        find_file_root(current_path)
    }
}

/// Returns the [PathBuf] of the directory containing the `Nargo.toml` by searching from `current_path` to the root of its [Path],
/// returning at the innermost directory found, i.e. the one corresponding to the package that contains the `current_path`.
///
/// Returns a [ManifestError] if no parent directories of `current_path` contain a manifest file.
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

/// Returns the [PathBuf] of the directory containing the `Nargo.toml` by searching from `current_path` to the root of its [Path],
/// returning at the topmost directory found, i.e. the one corresponding to the entire workspace.
///
/// Returns a [ManifestError] if no parent directories of `current_path` contain a manifest file.
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

/// Returns the [PathBuf] of the `Nargo.toml` file by searching from `current_path` and stopping at `root_path`.
///
/// Returns a [ManifestError] if no parent directories of `current_path` contain a manifest file.
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

/// Returns the [PathBuf] of the `Nargo.toml` file in the `current_path` directory.
///
/// Returns a [ManifestError] if `current_path` does not contain a manifest file.
pub fn get_package_manifest(current_path: &Path) -> Result<PathBuf, ManifestError> {
    let toml_path = current_path.join("Nargo.toml");
    if toml_path.exists() {
        Ok(toml_path)
    } else {
        Err(ManifestError::MissingFile(current_path.to_path_buf()))
    }
}

#[derive(Debug, Deserialize, Clone)]
struct PackageConfig {
    package: PackageMetadata,
    #[serde(default)]
    dependencies: BTreeMap<String, DependencyConfig>,
}

impl PackageConfig {
    fn resolve_to_package(
        &self,
        root_dir: &Path,
        processed: &mut Vec<String>,
    ) -> Result<Package, ManifestError> {
        let name: CrateName = if let Some(name) = &self.package.name {
            name.parse().map_err(|_| ManifestError::InvalidPackageName {
                toml: root_dir.join("Nargo.toml"),
                name: name.into(),
            })?
        } else {
            return Err(ManifestError::MissingNameField { toml: root_dir.join("Nargo.toml") });
        };

        let mut dependencies: BTreeMap<CrateName, Dependency> = BTreeMap::new();
        for (name, dep_config) in self.dependencies.iter() {
            let name = name.parse().map_err(|_| ManifestError::InvalidDependencyName {
                toml: root_dir.join("Nargo.toml"),
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
                    root_dir.join("Nargo.toml"),
                    invalid.to_string(),
                ))
            }
            None => return Err(ManifestError::MissingPackageType(root_dir.join("Nargo.toml"))),
        };

        let entry_path = if let Some(entry_path) = &self.package.entry {
            let custom_entry_path = root_dir.join(entry_path);
            if custom_entry_path.exists() {
                custom_entry_path
            } else {
                return Err(ManifestError::MissingEntryFile {
                    toml: root_dir.join("Nargo.toml"),
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

            if default_entry_path.exists() {
                default_entry_path
            } else {
                return Err(ManifestError::MissingDefaultEntryFile {
                    toml: root_dir.join("Nargo.toml"),
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

        let expression_width = self
            .package
            .expression_width
            .as_ref()
            .map(|expression_width| {
                parse_expression_width(expression_width)
                    .map_err(|err| ManifestError::ParseExpressionWidth(err.to_string()))
            })
            .map_or(Ok(None), |res| res.map(Some))?;

        Ok(Package {
            version: self.package.version.clone(),
            compiler_required_version: self.package.compiler_version.clone(),
            root_dir: root_dir.to_path_buf(),
            entry_path,
            package_type,
            name,
            dependencies,
            expression_width,
        })
    }
}

/// Contains all the information about a package, as loaded from a `Nargo.toml`.
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum Config {
    /// Represents a `Nargo.toml` with package fields.
    Package {
        #[serde(flatten)]
        package_config: PackageConfig,
    },
    /// Represents a `Nargo.toml` with workspace fields.
    Workspace {
        #[serde(alias = "workspace")]
        workspace_config: WorkspaceConfig,
    },
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

/// Tracks the root_dir of a `Nargo.toml` and the contents inside the file.
struct NargoToml {
    root_dir: PathBuf,
    config: Config,
}

#[derive(Default, Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
struct WorkspaceConfig {
    /// List of members in this workspace.
    members: Vec<PathBuf>,
    /// Specifies the default crate to interact with in the context (similarly to how we have nargo as the default crate in this repository).
    default_member: Option<PathBuf>,
}

#[allow(dead_code)]
#[derive(Default, Debug, Deserialize, Clone)]
struct PackageMetadata {
    name: Option<String>,
    version: Option<String>,
    #[serde(alias = "type")]
    package_type: Option<String>,
    entry: Option<PathBuf>,
    description: Option<String>,
    authors: Option<Vec<String>>,
    // If no compiler version is supplied, the latest is used
    // For now, we state that all packages must be compiled under the same
    // compiler version.
    // We also state that ACIR and the compiler will upgrade in lockstep.
    // so you will not need to supply an ACIR and compiler version
    compiler_version: Option<String>,
    license: Option<String>,
    expression_width: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
/// Enum representing the different types of ways to
/// supply a source for the dependency
enum DependencyConfig {
    Github { git: String, tag: String, directory: Option<String> },
    Path { path: String },
}

impl DependencyConfig {
    fn resolve_to_dependency(
        &self,
        pkg_root: &Path,
        processed: &mut Vec<String>,
    ) -> Result<Dependency, ManifestError> {
        let dep = match self {
            Self::Github { git, tag, directory } => {
                let dir_path = clone_git_repo(git, tag).map_err(ManifestError::GitError)?;
                let project_path = if let Some(directory) = directory {
                    let internal_path = dir_path.join(directory).normalize();
                    if !internal_path.starts_with(&dir_path) {
                        return Err(ManifestError::InvalidDirectory {
                            toml: pkg_root.join("Nargo.toml"),
                            directory: directory.into(),
                        });
                    }
                    internal_path
                } else {
                    dir_path
                };
                let toml_path = project_path.join("Nargo.toml");
                let package = resolve_package_from_toml(&toml_path, processed)?;
                Dependency::Remote { package }
            }
            Self::Path { path } => {
                let dir_path = pkg_root.join(path);
                let toml_path = dir_path.join("Nargo.toml");
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

fn toml_to_workspace(
    nargo_toml: NargoToml,
    package_selection: PackageSelection,
) -> Result<Workspace, ManifestError> {
    let mut resolved = Vec::new();
    let workspace = match nargo_toml.config {
        Config::Package { package_config } => {
            let member = package_config.resolve_to_package(&nargo_toml.root_dir, &mut resolved)?;
            match &package_selection {
                PackageSelection::Selected(selected_name) if selected_name != &member.name => {
                    return Err(ManifestError::MissingSelectedPackage(member.name))
                }
                _ => Workspace {
                    root_dir: nargo_toml.root_dir,
                    selected_package_index: Some(0),
                    members: vec![member],
                    is_assumed: false,
                },
            }
        }
        Config::Workspace { workspace_config } => {
            let mut members = Vec::new();
            let mut selected_package_index = None;
            for (index, member_path) in workspace_config.members.into_iter().enumerate() {
                let package_root_dir = nargo_toml.root_dir.join(&member_path);
                let package_toml_path = package_root_dir.join("Nargo.toml");
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
            }
        }
    };

    Ok(workspace)
}

fn read_toml(toml_path: &Path) -> Result<NargoToml, ManifestError> {
    let toml_path = toml_path.normalize();
    let toml_as_string = std::fs::read_to_string(&toml_path)
        .map_err(|_| ManifestError::ReadFailed(toml_path.to_path_buf()))?;
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
    // Checks for cyclic dependencies
    let str_path = toml_path.to_str().expect("ICE - path is empty");
    if processed.contains(&str_path.to_string()) {
        let mut cycle = false;
        let mut message = String::new();
        for toml in processed {
            cycle = cycle || toml == str_path;
            if cycle {
                message += &format!("{} referencing ", toml);
            }
        }
        message += str_path;
        return Err(ManifestError::CyclicDependency { cycle: message });
    }
    // Adds the package to the set of resolved packages
    if let Some(str) = toml_path.to_str() {
        processed.push(str.to_string());
    }

    let nargo_toml = read_toml(toml_path)?;

    let result = match nargo_toml.config {
        Config::Package { package_config } => {
            package_config.resolve_to_package(&nargo_toml.root_dir, processed)
        }
        Config::Workspace { .. } => {
            Err(ManifestError::UnexpectedWorkspace(toml_path.to_path_buf()))
        }
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
pub fn resolve_workspace_from_toml(
    toml_path: &Path,
    package_selection: PackageSelection,
    current_compiler_version: Option<String>,
) -> Result<Workspace, ManifestError> {
    let nargo_toml = read_toml(toml_path)?;
    let workspace = toml_to_workspace(nargo_toml, package_selection)?;
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

    use crate::{find_root, Config, ManifestError};

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
                        "cannot increase indent by more than {INDENT_SIZE}; item = {item}, current_dir={}", current_dir.display()
                    );

                    // Go into the last created directory
                    if indent > current_indent && last_item.is_some() {
                        let last_item = last_item.unwrap();
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
}
