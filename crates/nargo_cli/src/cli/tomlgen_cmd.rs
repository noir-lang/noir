use crate::{
    errors::{CliError, FilesystemError},
    resolver::resolve_root_manifest,
};
use acvm::Backend;
use clap::Args;
use iter_extended::btree_map;
use noirc_abi::{AbiParameter, AbiType, MAIN_RETURN_NAME};
use noirc_driver::{compute_function_signature, CompileOptions};

use std::{
    fs::remove_file,
    path::{Path, PathBuf},
};

use super::NargoConfig;
use super::{check_cmd::check_crate_and_report_errors, fs::write_to_file};
use crate::constants::{PROVER_INPUT_FILE, VERIFIER_INPUT_FILE};

/// Generate Prover.toml and Verifier.toml
#[derive(Debug, Clone, Args)]
pub(crate) struct TomlgenCommand {
    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: TomlgenCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    tomlgen(backend, &config.program_dir, &args.compile_options)?;
    println!("Toml files successfully generated!");
    Ok(())
}

fn tomlgen<B: Backend>(
    _backend: &B,
    program_dir: &Path,
    compile_options: &CompileOptions,
) -> Result<(), CliError<B>> {
    let (mut context, crate_id) = resolve_root_manifest(program_dir, None)?;
    check_crate_and_report_errors(&mut context, crate_id, compile_options.deny_warnings)?;
    if let Some((parameters, return_type)) = compute_function_signature(&context, &crate_id) {
        // XXX: The root config should return an enum to determine if we are looking for .json or .toml
        // For now it is hard-coded to be toml.
        //
        // Check for input.toml and verifier.toml
        let path_to_root = PathBuf::from(program_dir);
        let path_to_prover_input = path_to_root.join(format!("{PROVER_INPUT_FILE}.toml"));
        let path_to_verifier_input = path_to_root.join(format!("{VERIFIER_INPUT_FILE}.toml"));

        // If they are not available, then create them and populate them based on the ABI
        clean_file(&path_to_prover_input)
            .map_err(|_| FilesystemError::CanNotRemoveFile(path_to_prover_input.clone()))?;
        let prover_toml = create_input_toml_template(parameters.clone(), None);
        write_to_file(prover_toml.as_bytes(), &path_to_prover_input);
        clean_file(&path_to_verifier_input)
            .map_err(|_| FilesystemError::CanNotRemoveFile(path_to_verifier_input.clone()))?;
        let public_inputs = parameters.into_iter().filter(|param| param.is_public()).collect();
        let verifier_toml = create_input_toml_template(public_inputs, return_type);
        write_to_file(verifier_toml.as_bytes(), &path_to_verifier_input);
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

pub(crate) fn clean_file<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    if std::fs::symlink_metadata(&path).is_err() {
        return Ok(());
    }

    remove_file(path)
}

#[cfg(test)]
mod tests {
    use noirc_abi::{AbiParameter, AbiType, AbiVisibility, Sign};

    use super::create_input_toml_template;
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
}
