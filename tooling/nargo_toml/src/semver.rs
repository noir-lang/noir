use crate::{errors::SemverError, ManifestError};
use nargo::{
    package::{Dependency, Package},
    workspace::Workspace,
};
use noirc_driver::CrateName;
use semver::{Error, Prerelease, Version, VersionReq};

// Parse a semver compatible version string
pub(crate) fn parse_semver_compatible_version(version: &str) -> Result<Version, Error> {
    let mut version = Version::parse(version)?;
    version.pre = Prerelease::EMPTY;
    Ok(version)
}

// Check that all of the packages in the workspace are compatible with the current compiler version
pub(crate) fn semver_check_workspace(
    workspace: &Workspace,
    current_compiler_version: String,
) -> Result<(), ManifestError> {
    let version = parse_semver_compatible_version(&current_compiler_version)
        .expect("The compiler version is not a valid semver version");
    for package in &workspace.members {
        semver_check_package(package, &version).map_err(ManifestError::SemverError)?;
    }

    Ok(())
}

// Check that a package and all of its dependencies are compatible with the current compiler version
fn semver_check_package(package: &Package, compiler_version: &Version) -> Result<(), SemverError> {
    // Check that this package's compiler version requirements are satisfied
    if let Some(version) = &package.compiler_required_version {
        let version_req = match VersionReq::parse(version) {
            Ok(version_req) => version_req,
            Err(err) => {
                return Err(SemverError::CouldNotParseRequiredVersion {
                    package_name: package.name.clone().into(),
                    error: err.to_string(),
                })
            }
        };

        validate_compiler_version_requirement(&package.name, &version_req)?;

        if !version_req.matches(compiler_version) {
            return Err(SemverError::IncompatibleVersion {
                package_name: package.name.clone(),
                required_compiler_version: version.clone(),
                compiler_version_found: strip_build_meta_data(compiler_version),
            });
        };
    }

    // Check that all of this package's dependencies' compiler version requirements are satisfied
    for dep in package.dependencies.values() {
        match dep {
            Dependency::Local { package } | Dependency::Remote { package } => {
                semver_check_package(package, compiler_version)?;
            }
        }
    }

    Ok(())
}

fn validate_compiler_version_requirement(
    package_name: &CrateName,
    required_compiler_version: &VersionReq,
) -> Result<(), SemverError> {
    if required_compiler_version.comparators.iter().any(|comparator| !comparator.pre.is_empty()) {
        return Err(SemverError::InvalidCompilerVersionRequirement {
            package_name: package_name.clone(),
            required_compiler_version: required_compiler_version.to_string(),
        });
    }

    Ok(())
}

// Strip the build meta data from the version string since it is ignored by semver.
fn strip_build_meta_data(version: &Version) -> String {
    let version_string = version.to_string();
    let mut split = version_string.split('+');
    split.next().expect("split was called on an empty string").to_string()
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf, str::FromStr};

    use nargo::package::PackageType;
    use noirc_frontend::graph::CrateName;

    use super::*;

    #[test]
    fn test_semver_check_smoke() {
        let compiler_version = Version::parse("0.1.0").unwrap();

        let mut package = Package {
            compiler_required_version: Some("0.1.0".to_string()),
            root_dir: PathBuf::new(),
            package_type: PackageType::Library,
            entry_path: PathBuf::new(),
            name: CrateName::from_str("test").unwrap(),
            dependencies: BTreeMap::new(),
            version: Some("1.0".to_string()),
            expression_width: None,
        };
        if let Err(err) = semver_check_package(&package, &compiler_version) {
            panic!("semver check should have passed. compiler version is 0.1.0 and required version from the package is 0.1.0\n error: {err:?}")
        };

        package.compiler_required_version = Some("0.2.0".to_string());
        let got_err = match semver_check_package(&package, &compiler_version) {
            Ok(_) => panic!("semver check should have failed. compiler version is 0.1.0 and required version from the package is 0.2.0"),
            Err(err) => err,
        };

        let expected_version_error = SemverError::IncompatibleVersion {
            package_name: CrateName::from_str("test").unwrap(),
            required_compiler_version: "0.2.0".to_string(),
            compiler_version_found: "0.1.0".to_string(),
        };
        assert_eq!(got_err, expected_version_error);
    }

    #[test]
    fn test_semver_dependency_check_smoke() {
        let compiler_version = Version::parse("0.1.0").unwrap();

        let mut package = Package {
            compiler_required_version: Some("0.1.0".to_string()),
            root_dir: PathBuf::new(),
            package_type: PackageType::Library,
            entry_path: PathBuf::new(),
            name: CrateName::from_str("test").unwrap(),
            dependencies: BTreeMap::new(),
            version: Some("1.0".to_string()),
            expression_width: None,
        };

        let valid_dependency = Package {
            compiler_required_version: Some("0.1.0".to_string()),
            root_dir: PathBuf::new(),
            package_type: PackageType::Library,
            entry_path: PathBuf::new(),
            name: CrateName::from_str("good_dependency").unwrap(),
            dependencies: BTreeMap::new(),
            version: Some("1.0".to_string()),
            expression_width: None,
        };
        let invalid_dependency = Package {
            compiler_required_version: Some("0.2.0".to_string()),
            root_dir: PathBuf::new(),
            package_type: PackageType::Library,
            entry_path: PathBuf::new(),
            name: CrateName::from_str("bad_dependency").unwrap(),
            dependencies: BTreeMap::new(),
            version: Some("1.0".to_string()),
            expression_width: None,
        };

        package.dependencies.insert(
            CrateName::from_str("test_dep_valid").unwrap(),
            Dependency::Local { package: valid_dependency.clone() },
        );

        if let Err(err) = semver_check_package(&package, &compiler_version) {
            panic!("semver check should have passed. compiler version is 0.1.0 and required version from the package is 0.1.0\n error: {err:?}")
        };

        package.dependencies.insert(
            CrateName::from_str("test_dep_invalid").unwrap(),
            Dependency::Local { package: invalid_dependency.clone() },
        );
        let got_err = match semver_check_package(&package,&compiler_version) {
            Ok(_) => panic!("semver check should have failed. compiler version is 0.1.0 and required version from the package is 0.2.0"),
            Err(err) => err,
        };

        let expected_version_error = SemverError::IncompatibleVersion {
            package_name: CrateName::from_str("bad_dependency").unwrap(),
            required_compiler_version: "0.2.0".to_string(),
            compiler_version_found: "0.1.0".to_string(),
        };
        assert_eq!(got_err, expected_version_error);
    }

    #[test]
    fn test_semver_carrot() {
        let compiler_version = Version::parse("0.2.0").unwrap();

        let package = Package {
            compiler_required_version: Some(">=0.1.0".to_string()),
            root_dir: PathBuf::new(),
            package_type: PackageType::Library,
            entry_path: PathBuf::new(),
            name: CrateName::from_str("test").unwrap(),
            dependencies: BTreeMap::new(),
            version: Some("1.0".to_string()),
            expression_width: None,
        };

        if let Err(err) = semver_check_package(&package, &compiler_version) {
            panic!("semver check should have passed. compiler version is 0.2.0 and required version from the package is >=0.1.0\n error: {err:?}")
        };
    }

    #[test]
    fn test_semver_prerelease() {
        let compiler_version = parse_semver_compatible_version("1.0.0-beta.0").unwrap();

        let package = Package {
            compiler_required_version: Some(">=0.1.0".to_string()),
            root_dir: PathBuf::new(),
            package_type: PackageType::Library,
            entry_path: PathBuf::new(),
            name: CrateName::from_str("test").unwrap(),
            dependencies: BTreeMap::new(),
            version: Some("1.0".to_string()),
            expression_width: None,
        };

        if let Err(err) = semver_check_package(&package, &compiler_version) {
            panic!("{err}");
        };
    }

    #[test]
    fn test_semver_build_data() {
        let compiler_version = Version::parse("0.1.0+this-is-ignored-by-semver").unwrap();

        let package = Package {
            compiler_required_version: Some("0.1.0".to_string()),
            root_dir: PathBuf::new(),
            package_type: PackageType::Library,
            entry_path: PathBuf::new(),
            name: CrateName::from_str("test").unwrap(),
            dependencies: BTreeMap::new(),
            version: Some("1.0".to_string()),
            expression_width: None,
        };

        if let Err(err) = semver_check_package(&package, &compiler_version) {
            panic!("semver check should have passed. compiler version is 0.1.0+build_data and required version from the package is 0.1.0\n The build data should be ignored\n error: {err:?}")
        };
    }
}
