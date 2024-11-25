use std::path::PathBuf;

use nargo::package::PackageType;
use noirc_frontend::graph::CrateName;
use thiserror::Error;

/// Errors covering situations where a package is either missing, malformed or does not pass semver
/// validation checks.
#[derive(Debug, Error)]
pub enum ManifestError {
    /// Package doesn't have a manifest file
    #[error("cannot find a Nargo.toml for {0}")]
    MissingFile(PathBuf),

    #[error("Cannot read file {0} - does it exist?")]
    ReadFailed(PathBuf),

    #[error("Nargo.toml is missing a parent directory")]
    MissingParent,

    #[error("Missing `type` field in {0}")]
    MissingPackageType(PathBuf),

    #[error("Cannot use `{1}` for `type` field in {0}")]
    InvalidPackageType(PathBuf, String),

    /// Package manifest is unreadable.
    #[error("Nargo.toml is badly formed, could not parse.\n\n {0}")]
    MalformedFile(#[from] toml::de::Error),

    #[error("Unexpected workspace definition found in {0}. If you're attempting to load this as a dependency, you may need to add a `directory` field to your `Nargo.toml` to show which package within the workspace to use")]
    UnexpectedWorkspace(PathBuf),

    #[error("Cannot find file {entry} which was specified as the `entry` field in {toml}")]
    MissingEntryFile { toml: PathBuf, entry: PathBuf },

    #[error(
        r#"Cannot find file {entry} which is defaulted due to specifying `type = "{package_type}"` in {toml}"#
    )]
    MissingDefaultEntryFile { toml: PathBuf, entry: PathBuf, package_type: PackageType },

    #[error("{} found in {toml}", if name.is_empty() { "Empty package name".into() } else { format!("Invalid package name `{name}`") })]
    InvalidPackageName { toml: PathBuf, name: String },

    #[error("{} found in {toml}", if name.is_empty() { "Empty dependency name".into() } else { format!("Invalid dependency name `{name}`") })]
    InvalidDependencyName { toml: PathBuf, name: String },

    #[error("Invalid directory path {directory} in {toml}: It must point to a subdirectory")]
    InvalidDirectory { toml: PathBuf, directory: PathBuf },

    /// Encountered error while downloading git repository.
    #[error("{0}")]
    GitError(String),

    #[error("Selected package `{0}` was not found")]
    MissingSelectedPackage(CrateName),

    #[error("Default package was not found. Does {0} exist in your workspace?")]
    MissingDefaultPackage(PathBuf),

    #[error("Package `{0}` has type `bin` but you cannot depend on binary packages")]
    BinaryDependency(CrateName),

    #[error("Missing `name` field in {toml}")]
    MissingNameField { toml: PathBuf },

    #[error("No common ancestor between {root} and {current}")]
    NoCommonAncestor { root: PathBuf, current: PathBuf },

    #[error(transparent)]
    SemverError(SemverError),

    #[error("Cyclic package dependency found when processing {cycle}")]
    CyclicDependency { cycle: String },

    #[error("Failed to parse expression width with the following error: {0}")]
    ParseExpressionWidth(String),
}

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum SemverError {
    #[error("Invalid value for `compiler_version` in package {package_name}. Requirements may only refer to full releases")]
    InvalidCompilerVersionRequirement { package_name: CrateName, required_compiler_version: String },
    #[error("Incompatible compiler version in package {package_name}. Required compiler version is {required_compiler_version} but the compiler version is {compiler_version_found}.\n Update the compiler_version field in Nargo.toml to >={required_compiler_version} or compile this project with version {required_compiler_version}")]
    IncompatibleVersion {
        package_name: CrateName,
        required_compiler_version: String,
        compiler_version_found: String,
    },
    #[error("Could not parse the required compiler version for package {package_name} in Nargo.toml. Error: {error}")]
    CouldNotParseRequiredVersion { package_name: String, error: String },
    #[error("Could not parse the package version for package {package_name} in Nargo.toml. Error: {error}")]
    CouldNotParsePackageVersion { package_name: String, error: String },
}
