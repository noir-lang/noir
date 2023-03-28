use std::path::Path;

use nargo_core::manifest::{parse_toml_str, InvalidPackageError, PackageManifest};

/// Parses a Nargo.toml file from it's path
/// The path to the toml file must be present.
/// Calling this function without this guarantee is an ICE.
pub(crate) fn parse<P: AsRef<Path>>(
    path_to_toml: P,
) -> Result<PackageManifest, InvalidPackageError> {
    let toml_as_string =
        std::fs::read_to_string(&path_to_toml).expect("ice: path given for toml file is invalid");

    parse_toml_str(&toml_as_string)
}
