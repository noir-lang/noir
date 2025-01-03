use crate::errors::CliError;

use clap::Args;
use fm::FileManager;
use iter_extended::btree_map;
use nargo::{
    errors::CompileError, insert_all_files_for_workspace_into_file_manager, ops::report_errors,
    package::Package, parse_all, prepare_package,
};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml};
use noirc_abi::{AbiParameter, AbiType, MAIN_RETURN_NAME};
use noirc_driver::{
    check_crate, compute_function_abi, CompileOptions, CrateId, NOIR_ARTIFACT_VERSION_STRING,
};
use noirc_frontend::hir::{Context, ParsedFiles};

use super::NargoConfig;
use super::{fs::write_to_file, PackageOptions};

/// Checks the constraint system for errors
#[derive(Debug, Clone, Args)]
#[clap(visible_alias = "c")]
pub(crate) struct CheckCommand {
    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    /// Force overwrite of existing files
    #[clap(long = "overwrite")]
    allow_overwrite: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(args: CheckCommand, config: NargoConfig) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let selection = args.package_options.package_selection();
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;

    let mut workspace_file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = parse_all(&workspace_file_manager);

    for package in &workspace {
        let any_file_written = check_package(
            &workspace_file_manager,
            &parsed_files,
            package,
            &args.compile_options,
            args.allow_overwrite,
        )?;
        if any_file_written {
            println!("[{}] Constraint system successfully built!", package.name);
        }
    }
    Ok(())
}

/// Evaluates the necessity to create or update Prover.toml and Verifier.toml based on the allow_overwrite flag and files' existence.
/// Returns `true` if any file was generated or updated, `false` otherwise.
fn check_package(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    compile_options: &CompileOptions,
    allow_overwrite: bool,
) -> Result<bool, CompileError> {
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);
    check_crate_and_report_errors(&mut context, crate_id, compile_options)?;

    if package.is_library() || package.is_contract() {
        // Libraries do not have ABIs while contracts have many, so we cannot generate a `Prover.toml` file.
        Ok(false)
    } else {
        // XXX: We can have a --overwrite flag to determine if you want to overwrite the Prover/Verifier.toml files
        if let Some((parameters, _)) = compute_function_abi(&context, &crate_id) {
            let path_to_prover_input = package.prover_input_path();

            // Before writing the file, check if it exists and whether overwrite is set
            let should_write_prover = !path_to_prover_input.exists() || allow_overwrite;

            if should_write_prover {
                let prover_toml = create_input_toml_template(parameters.clone(), None);
                write_to_file(prover_toml.as_bytes(), &path_to_prover_input);
            } else {
                eprintln!("Note: Prover.toml already exists. Use --overwrite to force overwrite.");
            }

            let any_file_written = should_write_prover;

            Ok(any_file_written)
        } else {
            Err(CompileError::MissingMainFunction(package.name.clone()))
        }
    }
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
            AbiType::Struct { fields, .. } => {
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

/// Run the lexing, parsing, name resolution, and type checking passes and report any warnings
/// and errors found.
pub(crate) fn check_crate_and_report_errors(
    context: &mut Context,
    crate_id: CrateId,
    options: &CompileOptions,
) -> Result<(), CompileError> {
    let result = check_crate(context, crate_id, options);
    report_errors(result, &context.file_manager, options.deny_warnings, options.silence_warnings)
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
                    path: String::from("MyStruct"),
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
