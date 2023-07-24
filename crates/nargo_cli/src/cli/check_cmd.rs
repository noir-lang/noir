use crate::{errors::CliError, resolver::resolve_root_manifest};
use acvm::Backend;
use clap::Args;
use iter_extended::btree_map;
use noirc_abi::{AbiParameter, AbiType, MAIN_RETURN_NAME};
use noirc_driver::{check_crate, compute_function_signature, CompileOptions};
use noirc_errors::reporter::ReportedErrors;
use noirc_frontend::{graph::CrateId, hir::Context};
use std::path::{Path, PathBuf};

use super::fs::write_to_file;
use super::NargoConfig;
use crate::constants::{PROVER_INPUT_FILE, VERIFIER_INPUT_FILE};

/// Checks the constraint system for errors
#[derive(Debug, Clone, Args)]
pub(crate) struct CheckCommand {
    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: CheckCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    check_from_path(backend, &config.program_dir, &args.compile_options)?;
    println!("Constraint system successfully built!");
    Ok(())
}

fn check_from_path<B: Backend>(
    // Backend isn't used but keeping it in the signature allows for better type inference
    // TODO: This function doesn't need to exist but requires a little more refactoring
    _backend: &B,
    program_dir: &Path,
    compile_options: &CompileOptions,
) -> Result<(), CliError<B>> {
    let (mut context, crate_id) = resolve_root_manifest(program_dir, None)?;
    check_crate_and_report_errors(
        &mut context,
        crate_id,
        compile_options.deny_warnings,
        compile_options.experimental_ssa,
    )?;

    // XXX: We can have a --overwrite flag to determine if you want to overwrite the Prover/Verifier.toml files
    if let Some((parameters, return_type)) = compute_function_signature(&context, &crate_id) {
        // XXX: The root config should return an enum to determine if we are looking for .json or .toml
        // For now it is hard-coded to be toml.
        //
        // Check for input.toml and verifier.toml
        let path_to_root = PathBuf::from(program_dir);
        let path_to_prover_input = path_to_root.join(format!("{PROVER_INPUT_FILE}.toml"));
        let path_to_verifier_input = path_to_root.join(format!("{VERIFIER_INPUT_FILE}.toml"));

        // If they are not available, then create them and populate them based on the ABI
        if !path_to_prover_input.exists() {
            let prover_toml = create_input_toml_template(parameters.clone(), None);
            write_to_file(prover_toml.as_bytes(), &path_to_prover_input);
        }
        if !path_to_verifier_input.exists() {
            let public_inputs = parameters.into_iter().filter(|param| param.is_public()).collect();

            let verifier_toml = create_input_toml_template(public_inputs, return_type);
            write_to_file(verifier_toml.as_bytes(), &path_to_verifier_input);
        }
    } else {
        // This means that this is a library. Libraries do not have ABIs.
    }
    Ok(())
}

/// Generates the contents of a toml file with fields for each of the passed parameters.
fn create_input_toml_template(
    parameters: Vec<AbiParameter>,
    return_type: Option<AbiType>,
) -> String {
    /// Returns a default placeholder `toml::Value` for `typ` which
    /// complies with the structure of the specified `AbiType`.
    fn default_value(typ: AbiType) -> toml::Value {
        match typ {
            AbiType::Array { length, typ } => {
                let default_value_vec = std::iter::repeat(default_value(*typ))
                    .take(length.try_into().unwrap())
                    .collect();
                toml::Value::Array(default_value_vec)
            }
            AbiType::Struct { fields } => {
                let default_value_map = toml::map::Map::from_iter(
                    fields.into_iter().map(|(name, typ)| (name, default_value(typ))),
                );
                toml::Value::Table(default_value_map)
            }
            _ => toml::Value::String("".to_owned()),
        }
    }

    let mut map =
        btree_map(parameters, |AbiParameter { name, typ, .. }| (name, default_value(typ)));

    if let Some(typ) = return_type {
        map.insert(MAIN_RETURN_NAME.to_owned(), default_value(typ));
    }

    toml::to_string(&map).unwrap()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use noirc_abi::{AbiParameter, AbiType, AbiVisibility, Sign};
    use noirc_driver::CompileOptions;

    use super::create_input_toml_template;

    const TEST_DATA_DIR: &str = "tests/target_tests_data";

    #[test]
    fn valid_toml_template() {
        let typed_param = |name: &str, typ: AbiType| AbiParameter {
            name: name.to_string(),
            typ,
            visibility: AbiVisibility::Public,
        };
        let parameters = vec![
            typed_param("a", AbiType::Field),
            typed_param("b", AbiType::Integer { sign: Sign::Unsigned, width: 32 }),
            typed_param("c", AbiType::Array { length: 2, typ: Box::new(AbiType::Field) }),
            typed_param(
                "d",
                AbiType::Struct {
                    fields: vec![
                        (String::from("d1"), AbiType::Field),
                        (
                            String::from("d2"),
                            AbiType::Array { length: 3, typ: Box::new(AbiType::Field) },
                        ),
                    ],
                },
            ),
            typed_param("e", AbiType::Boolean),
        ];

        let toml_str = create_input_toml_template(parameters, None);

        let expected_toml_str = r#"a = ""
b = ""
c = ["", ""]
e = ""

[d]
d1 = ""
d2 = ["", "", ""]
"#;
        assert_eq!(toml_str, expected_toml_str);
    }

    #[test]
    fn pass() {
        let pass_dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("{TEST_DATA_DIR}/pass"));

        let backend = crate::backends::ConcreteBackend::default();
        let config = CompileOptions::default();
        let paths = std::fs::read_dir(pass_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(
                super::check_from_path(&backend, &path, &config).is_ok(),
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

        let backend = crate::backends::ConcreteBackend::default();
        let config = CompileOptions::default();
        let paths = std::fs::read_dir(fail_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(
                super::check_from_path(&backend, &path, &config).is_err(),
                "path: {}",
                path.display()
            );
        }
    }

    #[test]
    fn pass_with_warnings() {
        let pass_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("{TEST_DATA_DIR}/pass_dev_mode"));

        let backend = crate::backends::ConcreteBackend::default();
        let config = CompileOptions { deny_warnings: false, ..Default::default() };

        let paths = std::fs::read_dir(pass_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(
                super::check_from_path(&backend, &path, &config).is_ok(),
                "path: {}",
                path.display()
            );
        }
    }
}

/// Run the lexing, parsing, name resolution, and type checking passes and report any warnings
/// and errors found.
pub(crate) fn check_crate_and_report_errors(
    context: &mut Context,
    crate_id: CrateId,
    deny_warnings: bool,
    experimental_ssa: bool,
) -> Result<(), ReportedErrors> {
    let result = check_crate(context, crate_id, deny_warnings, experimental_ssa)
        .map(|warnings| ((), warnings));
    super::compile_cmd::report_errors(result, context, deny_warnings)
}
