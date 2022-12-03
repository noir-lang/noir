use crate::{errors::CliError, resolver::Resolver};
use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use noirc_abi::{Abi, AbiType};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use super::{add_std_lib, write_to_file, PROVER_INPUT_FILE, VERIFIER_INPUT_FILE};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("build").unwrap();
    let allow_warnings = args.is_present("allow-warnings");

    let package_dir = std::env::current_dir().unwrap();
    build_from_path(package_dir, allow_warnings)?;
    println!("Constraint system successfully built!");
    Ok(())
}
// This is exposed so that we can run the examples and verify that they pass
pub fn build_from_path<P: AsRef<Path>>(p: P, allow_warnings: bool) -> Result<(), CliError> {
    let backend = crate::backends::ConcreteBackend;
    let mut driver = Resolver::resolve_root_config(p.as_ref(), backend.np_language())?;
    add_std_lib(&mut driver);
    driver.build(allow_warnings);
    // XXX: We can have a --overwrite flag to determine if you want to overwrite the Prover/Verifier.toml files
    if let Some(abi) = driver.compute_abi() {
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
            let toml = toml::to_string(&build_empty_map(&abi)).unwrap();
            write_to_file(toml.as_bytes(), &path_to_prover_input);
        }
        if !path_to_verifier_input.exists() {
            let public_abi = abi.public_abi();
            let toml = toml::to_string(&build_empty_map(&public_abi)).unwrap();
            write_to_file(toml.as_bytes(), &path_to_verifier_input);
        }
    } else {
        // This means that this is a library. Libraries do not have ABIs.
    }
    Ok(())
}

fn build_empty_map(abi: &Abi) -> BTreeMap<String, &str> {
    abi.parameters
        .iter()
        .map(|(param_name, param_type)| {
            let default_value = if matches!(param_type, AbiType::Array { .. }) { "[]" } else { "" };
            (param_name.to_string(), default_value)
        })
        .collect()
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
            assert!(
                super::build_from_path(path.clone(), false).is_ok(),
                "path: {}",
                path.display()
            );
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
            assert!(
                super::build_from_path(path.clone(), false).is_err(),
                "path: {}",
                path.display()
            );
        }
    }

    #[test]
    fn pass_with_warnings() {
        let mut pass_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        pass_dir.push(&format!("{TEST_DATA_DIR}/pass_dev_mode"));

        let paths = std::fs::read_dir(pass_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(super::build_from_path(path.clone(), true).is_ok(), "path: {}", path.display());
        }
    }
}
