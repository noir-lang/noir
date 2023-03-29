use serde::Deserialize;
use std::collections::BTreeMap;

mod errors;
pub use self::errors::InvalidPackageError;

#[derive(Debug, Deserialize, Clone)]
pub struct PackageManifest {
    pub package: PackageMetadata,
    pub dependencies: BTreeMap<String, Dependency>,
}

impl PackageManifest {
    /// Returns whether the package has a local dependency.
    // Local paths are usually relative and are discouraged when sharing libraries
    // It is better to separate these into different packages.
    pub fn has_local_dependency(&self) -> bool {
        self.dependencies.values().any(|dep| matches!(dep, Dependency::Path { .. }))
    }

    pub fn from_toml_str(toml_as_string: &str) -> Result<Self, InvalidPackageError> {
        let manifest = toml::from_str::<PackageManifest>(toml_as_string)?;
        Ok(manifest)
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct PackageMetadata {
    // Note: a package name is not needed unless there is a registry
    authors: Vec<String>,
    // If not compiler version is supplied, the latest is used
    // For now, we state that all packages must be compiled under the same
    // compiler version.
    // We also state that ACIR and the compiler will upgrade in lockstep.
    // so you will not need to supply an ACIR and compiler version
    compiler_version: Option<String>,
    backend: Option<String>,
    license: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
/// Enum representing the different types of ways to
/// supply a source for the dependency
pub enum Dependency {
    Github { git: String, tag: String },
    Path { path: String },
}

#[test]
fn parse_standard_toml() {
    let src = r#"

        [package]
        authors = ["kev", "foo"]
        compiler_version = "0.1"

        [dependencies]
        rand = { tag = "next", git = "https://github.com/rust-lang-nursery/rand"}
        cool = { tag = "next", git = "https://github.com/rust-lang-nursery/rand"}
        hello = {path = "./noir_driver"}
    "#;

    assert!(PackageManifest::from_toml_str(src).is_ok());
}
