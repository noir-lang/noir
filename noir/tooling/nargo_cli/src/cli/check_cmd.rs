use crate::backends::Backend;
use crate::errors::CliError;

use clap::Args;
use fm::FileManager;
use iter_extended::btree_map;
use nargo::{
    errors::CompileError, insert_all_files_for_workspace_into_file_manager, package::Package,
    parse_all, prepare_package,
};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_abi::{AbiParameter, AbiType, MAIN_RETURN_NAME};
use noirc_driver::{
    check_crate, compute_function_abi, file_manager_with_stdlib, CompileOptions,
    NOIR_ARTIFACT_VERSION_STRING,
};
use noirc_frontend::{
    graph::{CrateId, CrateName},
    hir::{Context, ParsedFiles},
};

use super::fs::write_to_file;
use super::NargoConfig;

/// Checks the constraint system for errors
#[derive(Debug, Clone, Args)]
pub(crate) struct CheckCommand {
    /// The name of the package to check
    #[clap(long, conflicts_with = "workspace")]
    package: Option<CrateName>,

    /// Check all packages in the workspace
    #[clap(long, conflicts_with = "package")]
    workspace: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(
    _backend: &Backend,
    args: CheckCommand,
    config: NargoConfig,
) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;

    let mut workspace_file_manager = file_manager_with_stdlib(&workspace.root_dir);
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = parse_all(&workspace_file_manager);

    for package in &workspace {
        check_package(&workspace_file_manager, &parsed_files, package, &args.compile_options)?;
        println!("[{}] Constraint system successfully built!", package.name);
    }
    Ok(())
}

fn check_package(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    compile_options: &CompileOptions,
) -> Result<(), CompileError> {
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);
    check_crate_and_report_errors(
        &mut context,
        crate_id,
        compile_options.deny_warnings,
        compile_options.disable_macros,
        compile_options.silence_warnings,
    )?;

    if package.is_library() || package.is_contract() {
        // Libraries do not have ABIs while contracts have many, so we cannot generate a `Prover.toml` file.
        Ok(())
    } else {
        // XXX: We can have a --overwrite flag to determine if you want to overwrite the Prover/Verifier.toml files
        if let Some((parameters, return_type)) = compute_function_abi(&context, &crate_id) {
            let path_to_prover_input = package.prover_input_path();
            let path_to_verifier_input = package.verifier_input_path();

            // If they are not available, then create them and populate them based on the ABI
            if !path_to_prover_input.exists() {
                let prover_toml = create_input_toml_template(parameters.clone(), None);
                write_to_file(prover_toml.as_bytes(), &path_to_prover_input);
            }
            if !path_to_verifier_input.exists() {
                let public_inputs =
                    parameters.into_iter().filter(|param| param.is_public()).collect();

                let verifier_toml = create_input_toml_template(public_inputs, return_type);
                write_to_file(verifier_toml.as_bytes(), &path_to_verifier_input);
            }

            Ok(())
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

/// Run the lexing, parsing, name resolution, and type checking passes and report any warnings
/// and errors found.
pub(crate) fn check_crate_and_report_errors(
    context: &mut Context,
    crate_id: CrateId,
    deny_warnings: bool,
    disable_macros: bool,
    silence_warnings: bool,
) -> Result<(), CompileError> {
    let result = check_crate(context, crate_id, deny_warnings, disable_macros);
    super::compile_cmd::report_errors(
        result,
        &context.file_manager,
        deny_warnings,
        silence_warnings,
    )
}
