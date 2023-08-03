use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use fm::{NormalizePath, FILE_EXTENSION};
use nargo::{
    package::{Dependency, Package, PackageType},
    workspace::Workspace,
};
use noirc_frontend::graph::CrateName;
use serde::Deserialize;

use crate::{errors::ManifestError, git::clone_git_repo};

#[derive(Debug, Deserialize, Clone)]
struct PackageConfig {
    package: PackageMetadata,
    #[serde(default)]
    dependencies: BTreeMap<String, DependencyConfig>,
}

impl PackageConfig {
    fn resolve_to_package(&self, root_dir: &Path) -> Result<Package, ManifestError> {
        let name = self.package.name.parse().map_err(|_| ManifestError::InvalidPackageName)?;

        let mut dependencies: BTreeMap<CrateName, Dependency> = BTreeMap::new();
        for (name, dep_config) in self.dependencies.iter() {
            let name = name.parse().map_err(|_| ManifestError::InvalidPackageName)?;
            let resolved_dep = dep_config.resolve_to_dependency(root_dir)?;

            dependencies.insert(name, resolved_dep);
        }

        let package_type = match self.package.package_type.as_deref() {
            Some("lib") => PackageType::Library,
            Some("bin") => PackageType::Binary,
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
                PackageType::Binary => {
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

        Ok(Package {
            root_dir: root_dir.to_path_buf(),
            entry_path,
            package_type,
            name,
            dependencies,
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
    #[serde(default = "panic_missing_name")]
    name: String,
    #[serde(alias = "type")]
    package_type: Option<String>,
    entry: Option<PathBuf>,
    description: Option<String>,
    authors: Option<Vec<String>>,
    // If not compiler version is supplied, the latest is used
    // For now, we state that all packages must be compiled under the same
    // compiler version.
    // We also state that ACIR and the compiler will upgrade in lockstep.
    // so you will not need to supply an ACIR and compiler version
    compiler_version: Option<String>,
    backend: Option<String>,
    license: Option<String>,
}

// TODO: Remove this after a couple of breaking releases (added in 0.10.0)
fn panic_missing_name() -> String {
    panic!(
        r#"

Failed to parse `Nargo.toml`.

`Nargo.toml` now requires a "name" field for Noir packages.

```toml
[package]
name = "package_name"
```

Modify your `Nargo.toml` similarly to above and rerun the command.

"#
    )
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
/// Enum representing the different types of ways to
/// supply a source for the dependency
enum DependencyConfig {
    Github { git: String, tag: String },
    Path { path: String },
}

impl DependencyConfig {
    fn resolve_to_dependency(&self, pkg_root: &Path) -> Result<Dependency, ManifestError> {
        let dep = match self {
            Self::Github { git, tag } => {
                let dir_path = clone_git_repo(git, tag).map_err(ManifestError::GitError)?;
                let toml_path = dir_path.join("Nargo.toml");
                let package = resolve_package_from_toml(&toml_path)?;
                Dependency::Remote { package }
            }
            Self::Path { path } => {
                let dir_path = pkg_root.join(path);
                let toml_path = dir_path.join("Nargo.toml");
                let package = resolve_package_from_toml(&toml_path)?;
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
    selected_package: Option<CrateName>,
) -> Result<Workspace, ManifestError> {
    let workspace = match nargo_toml.config {
        Config::Package { package_config } => {
            let member = package_config.resolve_to_package(&nargo_toml.root_dir)?;
            if selected_package.is_none() || Some(&member.name) == selected_package.as_ref() {
                Workspace {
                    root_dir: nargo_toml.root_dir,
                    selected_package_index: Some(0),
                    members: vec![member],
                }
            } else {
                return Err(ManifestError::MissingSelectedPackage(member.name));
            }
        }
        Config::Workspace { workspace_config } => {
            let mut members = Vec::new();
            let mut selected_package_index = None;
            for (index, member_path) in workspace_config.members.into_iter().enumerate() {
                let package_root_dir = nargo_toml.root_dir.join(&member_path);
                let package_toml_path = package_root_dir.join("Nargo.toml");
                let member = resolve_package_from_toml(&package_toml_path)?;

                match selected_package.as_ref() {
                    Some(selected_name) => {
                        if &member.name == selected_name {
                            selected_package_index = Some(index);
                        }
                    }
                    None => {
                        if Some(&member_path) == workspace_config.default_member.as_ref() {
                            selected_package_index = Some(index);
                        }
                    }
                }

                members.push(member);
            }

            // If the selected_package_index is still `None` but we have see a default_member or selected package,
            // we want to present an error to users
            if selected_package_index.is_none() {
                if let Some(selected_name) = selected_package {
                    return Err(ManifestError::MissingSelectedPackage(selected_name));
                }
                if let Some(default_path) = workspace_config.default_member {
                    return Err(ManifestError::MissingDefaultPackage(default_path));
                }
            }

            Workspace { root_dir: nargo_toml.root_dir, members, selected_package_index }
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
fn resolve_package_from_toml(toml_path: &Path) -> Result<Package, ManifestError> {
    let nargo_toml = read_toml(toml_path)?;

    match nargo_toml.config {
        Config::Package { package_config } => {
            package_config.resolve_to_package(&nargo_toml.root_dir)
        }
        Config::Workspace { .. } => {
            Err(ManifestError::UnexpectedWorkspace(toml_path.to_path_buf()))
        }
    }
}

/// Resolves a Nargo.toml file into a `Workspace` struct as defined by our `nargo` core.
pub(crate) fn resolve_workspace_from_toml(
    toml_path: &Path,
    selected_package: Option<CrateName>,
) -> Result<Workspace, ManifestError> {
    let nargo_toml = read_toml(toml_path)?;

    toml_to_workspace(nargo_toml, selected_package)
}

#[test]
fn parse_standard_toml() {
    let src = r#"

        [package]
        name = "test"
        authors = ["kev", "foo"]
        compiler_version = "0.1"

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
        compiler_version = "0.1"
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
