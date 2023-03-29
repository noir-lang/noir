use std::path::PathBuf;
use thiserror::Error;

/// Errors covering situations where a package is either missing or malformed.
#[derive(Debug, Error)]
pub enum InvalidPackageError {
    /// Package doesn't have a manifest file
    #[error("cannot find a Nargo.toml in {}", .0.display())]
    MissingManifestFile(PathBuf),

    /// Package manifest is unreadable.
    #[error("Nargo.toml is badly formed, could not parse.\n\n {0}")]
    MalformedManifestFile(#[from] toml::de::Error),

    /// Package does not contain Noir source files.
    #[error("cannot find src directory in path {}", .0.display())]
    NoSourceDir(PathBuf),

    /// Package has neither of `main.nr` and `lib.nr`.
    #[error("package must contain either a `lib.nr`(Library) or a `main.nr`(Binary).")]
    ContainsZeroCrates,

    /// Package has both a `main.nr` (for binaries) and `lib.nr` (for libraries)
    #[error("package cannot contain both a `lib.nr` and a `main.nr`")]
    ContainsMultipleCrates,
}
