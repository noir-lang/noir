use clap::ArgMatches;
use std::path::{Path, PathBuf};
use acvm::ProofSystemCompiler;
use crate::{errors::CliError, resolver::Resolver};

use super::{add_std_lib, write_to_file, PROVER_INPUT_FILE, VERIFIER_INPUT_FILE};

pub(crate) fn run(_args: ArgMatches) -> Result<(), CliError> {
    let package_dir = std::env::current_dir().unwrap();
    build_from_path(package_dir)?;
    println!("Constraint system successfully built!");
    Ok(())
}
// This is exposed so that we can run the examples and verify that they pass
pub fn build_from_path<P: AsRef<Path>>(p: P) -> Result<(), CliError> {
    let backend = crate::backends::ConcreteBackend;
    let mut driver = Resolver::resolve_root_config(p.as_ref(), backend.np_language())?;
    add_std_lib(&mut driver);
    driver.build();
    // XXX: We can have a --overwrite flag to determine if you want to overwrite the Prover/Verifier.toml files
    if let Some(x) = driver.compute_abi() {
        // XXX: The root config should return an enum to determine if we are looking for .json or .toml
        // For now it is hard-coded to be toml.
        //
        // Check for input.toml and verifier.toml
        let path_to_root = PathBuf::from(p.as_ref());
        let path_to_prover_input = path_to_root.join(format!("{}.toml", PROVER_INPUT_FILE));
        let path_to_verifier_input = path_to_root.join(format!("{}.toml", VERIFIER_INPUT_FILE));

        // If they are not available, then create them and
        // populate them based on the ABI
        if !path_to_prover_input.exists() {
            let toml = toml::to_string(&x).unwrap();
            write_to_file(toml.as_bytes(), &path_to_prover_input);
        }
        if !path_to_verifier_input.exists() {
            let abi = x.public_abi();
            let toml = toml::to_string(&abi).unwrap();
            write_to_file(toml.as_bytes(), &path_to_verifier_input);
        }
    } else {
        // This means that this is a library. Libraries do not have ABIs.
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    const TEST_DATA_DIR: &str = "tests/build_tests_data";

    #[test]
    fn pass() {
        let mut pass_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        pass_dir.push(&format!("{TEST_DATA_DIR}/pass"));

        let paths = std::fs::read_dir(pass_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(super::build_from_path(path.clone()).is_ok(), "path: {}", path.display());
        }
    }

    #[test]
    #[ignore = "This test fails because the reporter exits the process with 1"]
    fn fail() {
        let mut fail_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        fail_dir.push(&format!("{TEST_DATA_DIR}/fail"));

        let paths = std::fs::read_dir(fail_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(super::build_from_path(path.clone()).is_err(), "path: {}", path.display());
        }
    }
}
