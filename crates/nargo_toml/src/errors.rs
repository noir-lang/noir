use std::path::PathBuf;

use nargo::package::PackageType;
use noirc_frontend::graph::CrateName;
use thiserror::Error;

/// Errors covering situations where a package is either missing or malformed.
#[derive(Debug, Error)]
pub enum ManifestError {
    /// Package doesn't have a manifest file
    #[error("cannot find a Nargo.toml in {}", .0.display())]
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

    #[error("Unxpected workspace definition found in {0}")]
    UnexpectedWorkspace(PathBuf),

    #[error("Cannot find file {entry} which was specified as the `entry` field in {toml}")]
    MissingEntryFile { toml: PathBuf, entry: PathBuf },

    #[error(
        r#"Cannot find file {entry} which is defaulted due to specifying `type = "{package_type}"` in {toml}"#
    )]
    MissingDefaultEntryFile { toml: PathBuf, entry: PathBuf, package_type: PackageType },

    /// Invalid character `-` in package name
    #[error("invalid character `-` in package name")]
    InvalidPackageName,

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
}
