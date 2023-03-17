use crate::{errors::CliError, resolver::Resolver};
use acvm::ProofSystemCompiler;
use clap::Args;
use iter_extended::btree_map;
use noirc_abi::{AbiParameter, AbiType, MAIN_RETURN_NAME};
use noirc_driver::CompileOptions;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use super::fs::write_to_file;
use super::{add_std_lib, NargoConfig};
use crate::constants::{PROVER_INPUT_FILE, VERIFIER_INPUT_FILE};

/// Checks the constraint system for errors
#[derive(Debug, Clone, Args)]
pub(crate) struct CheckCommand {
    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(args: CheckCommand, config: NargoConfig) -> Result<(), CliError> {
    check_from_path(config.program_dir, &args.compile_options)?;
    println!("Constraint system successfully built!");
    Ok(())
}

fn check_from_path<P: AsRef<Path>>(p: P, compile_options: &CompileOptions) -> Result<(), CliError> {
    let backend = crate::backends::ConcreteBackend;

    let mut driver = Resolver::resolve_root_config(p.as_ref(), backend.np_language())?;
    add_std_lib(&mut driver);

    driver.check_crate(compile_options).map_err(|_| CliError::CompilationError)?;

    // XXX: We can have a --overwrite flag to determine if you want to overwrite the Prover/Verifier.toml files
    if let Some((parameters, return_type)) = driver.compute_function_signature() {
        // XXX: The root config should return an enum to determine if we are looking for .json or .toml
        // For now it is hard-coded to be toml.
        //
        // Check for input.toml and verifier.toml
        let path_to_root = PathBuf::from(p.as_ref());
        let path_to_prover_input = path_to_root.join(format!("{PROVER_INPUT_FILE}.toml"));
        let path_to_verifier_input = path_to_root.join(format!("{VERIFIER_INPUT_FILE}.toml"));

        // If they are not available, then create them and
        // populate them based on the ABI
        if !path_to_prover_input.exists() {
            let toml =
                toml::to_string(&build_placeholder_input_map(parameters.clone(), None)).unwrap();
            write_to_file(toml.as_bytes(), &path_to_prover_input);
        }
        if !path_to_verifier_input.exists() {
            let public_inputs = parameters.into_iter().filter(|param| param.is_public()).collect();

            let toml =
                toml::to_string(&build_placeholder_input_map(public_inputs, return_type)).unwrap();
            write_to_file(toml.as_bytes(), &path_to_verifier_input);
        }
    } else {
        // This means that this is a library. Libraries do not have ABIs.
    }
    Ok(())
}

fn build_placeholder_input_map(
    parameters: Vec<AbiParameter>,
    return_type: Option<AbiType>,
) -> BTreeMap<String, &'static str> {
    let default_value = |typ: AbiType| {
        if matches!(typ, AbiType::Array { .. }) {
            "[]"
        } else {
            ""
        }
    };

    let mut map =
        btree_map(parameters, |AbiParameter { name, typ, .. }| (name, default_value(typ)));

    if let Some(typ) = return_type {
        map.insert(MAIN_RETURN_NAME.to_owned(), default_value(typ));
    }

    map
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use noirc_driver::CompileOptions;

    const TEST_DATA_DIR: &str = "tests/target_tests_data";

    #[test]
    fn pass() {
        let pass_dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("{TEST_DATA_DIR}/pass"));

        let config = CompileOptions::default();
        let paths = std::fs::read_dir(pass_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(
                super::check_from_path(path.clone(), &config).is_ok(),
                "path: {}",
                path.display()
            );
        }
    }

    #[test]
    #[ignore = "This test fails because the reporter exits the process with 1"]
    fn fail() {
        let fail_dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("{TEST_DATA_DIR}/fail"));

        let config = CompileOptions::default();
        let paths = std::fs::read_dir(fail_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(
                super::check_from_path(path.clone(), &config).is_err(),
                "path: {}",
                path.display()
            );
        }
    }

    #[test]
    fn pass_with_warnings() {
        let pass_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("{TEST_DATA_DIR}/pass_dev_mode"));

        let config = CompileOptions { allow_warnings: true, ..Default::default() };

        let paths = std::fs::read_dir(pass_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(
                super::check_from_path(path.clone(), &config).is_ok(),
                "path: {}",
                path.display()
            );
        }
    }
}
