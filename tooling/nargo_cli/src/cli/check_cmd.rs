use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;
use std::path::Path;

use crate::errors::CliError;

use clap::Args;
use fm::{FileId, FileManager};
use iter_extended::btree_map;
use nargo::{
    errors::CompileError,
    insert_all_files_for_workspace_into_file_manager,
    ops::{check_crate_and_report_errors, report_errors},
    package::Package,
    parse_all, prepare_package,
    workspace::Workspace,
};
use nargo_toml::PackageSelection;
use noir_artifact_cli::fs::artifact::write_to_file;
use noirc_abi::{AbiParameter, AbiType, MAIN_RETURN_NAME};
use noirc_driver::{
    CompileOptions, check_crate, check_crate_returning_frontend_errors, compute_function_abi,
};
use noirc_errors::{CustomDiagnostic, Location};
use noirc_frontend::{
    fix::{Fixes, apply_fixes},
    graph::CrateId,
    hir::resolution::errors::ResolverError,
    hir::{Context, ParsedFiles, def_collector::dc_crate::CompilationError as FrontendError},
    monomorphization::monomorphize,
};

use super::{LockType, PackageOptions, WorkspaceCommand};

/// Check a local package and all of its dependencies for errors
#[derive(Debug, Clone, Args)]
#[clap(visible_alias = "c")]
pub(crate) struct CheckCommand {
    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    /// Force overwrite of existing files
    #[clap(long)]
    overwrite: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,

    /// Just show the hash of each packages, without actually performing the check.
    #[clap(long, hide = true)]
    show_program_hash: bool,

    /// Rewrite the package's source files, fixing any warnings whose fix is a pure removal:
    /// unused imports are removed and unnecessary `mut` modifiers are dropped.
    /// Applying a fix can reveal further fixable warnings (e.g. an import only referenced by
    /// a removed import); run the command again to fix those too.
    #[clap(long, hide = true)]
    fix: bool,
}

impl WorkspaceCommand for CheckCommand {
    fn package_selection(&self) -> PackageSelection {
        self.package_options.package_selection()
    }
    fn lock_type(&self) -> LockType {
        // Creates a `Prover.toml` template if it doesn't exist, otherwise only writes if `allow_overwrite` is true,
        // so it shouldn't lead to accidental conflicts. Doesn't produce compilation artifacts.
        LockType::None
    }
}

pub(crate) fn run(args: CheckCommand, workspace: Workspace) -> Result<(), CliError> {
    let mut workspace_file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = parse_all(&workspace_file_manager);

    for package in &workspace {
        if args.show_program_hash {
            let (mut context, crate_id) =
                prepare_package(&workspace_file_manager, &parsed_files, package);
            check_crate(&mut context, crate_id, &args.compile_options).unwrap();
            let Some(main) = context.get_main_function(&crate_id) else {
                continue;
            };
            let program = monomorphize(
                main,
                &mut context.def_interner,
                context.file_manager.as_file_map(),
                false,
            )
            .unwrap();
            let hash = rustc_hash::FxBuildHasher.hash_one(&program);
            println!("{}: {:x}", package.name, hash);
            continue;
        }

        check_package(
            &workspace_file_manager,
            &parsed_files,
            package,
            &args.compile_options,
            args.overwrite,
            args.fix,
        )?;
    }
    Ok(())
}

/// Evaluates the necessity to create or update Prover.toml and Verifier.toml based on the `allow_overwrite` flag and files' existence.
/// Returns `true` if any file was generated or updated, `false` otherwise.
fn check_package(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    compile_options: &CompileOptions,
    overwrite: bool,
    fix: bool,
) -> Result<(), CliError> {
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);

    if fix {
        let (mut result, frontend_errors) =
            check_crate_returning_frontend_errors(&mut context, crate_id, compile_options);

        // Only rewrite sources when the check succeeded: with compilation errors present the
        // frontend's picture of the program is not reliable. Warnings for the code that was
        // just fixed are dropped so they aren't reported for code that no longer exists.
        if let Ok((_, warnings)) = &mut result {
            let fixed_locations =
                apply_fixes_to_package(&context, crate_id, parsed_files, &frontend_errors)?;
            remove_fixed_warnings(warnings, &fixed_locations);
        }

        report_errors(
            result,
            &context.file_manager,
            &context.parsed_files,
            compile_options.deny_warnings,
            compile_options.silence_warnings,
        )?;
    } else {
        check_crate_and_report_errors(&mut context, crate_id, compile_options)?;
    }

    if package.is_library() || package.is_contract() {
        // Libraries do not have ABIs while contracts have many, so we cannot generate a `Prover.toml` file.
        Ok(())
    } else if let Some((parameters, return_type)) = compute_function_abi(&context, &crate_id) {
        let path_to_prover_input = package.prover_input_path();

        // Before writing the file, check if it exists and whether overwrite is set
        let should_write_prover = !path_to_prover_input.exists() || overwrite;

        if should_write_prover {
            let prover_toml = create_input_toml_template(parameters, return_type);
            write_to_file(prover_toml.as_bytes(), &path_to_prover_input)
                .expect("failed to write template");
        } else {
            eprintln!("Note: Prover.toml already exists. Use --overwrite to force overwrite.");
        }

        Ok(())
    } else {
        Err(CompileError::MissingMainFunction(package.name.clone()).into())
    }
}

/// Rewrites the package's source files on disk, applying every removal-only fix the frontend
/// reported: unused imports are pruned and unnecessary `mut` modifiers are dropped. Only
/// files belonging to `crate_id` are touched, so dependencies are never modified. Returns the
/// locations of the fixed warnings (the imports that were removed, the bindings whose `mut`
/// was dropped).
fn apply_fixes_to_package(
    context: &Context,
    crate_id: CrateId,
    parsed_files: &ParsedFiles,
    frontend_errors: &[FrontendError],
) -> Result<HashSet<Location>, CliError> {
    let mut fixes_per_file: HashMap<FileId, Fixes> = HashMap::new();

    for (module_id, unused_imports) in context.usage_tracker.unused_imports() {
        if module_id.krate != crate_id {
            continue;
        }
        for (ident, location) in unused_imports.keys() {
            fixes_per_file
                .entry(location.file)
                .or_default()
                .unused_imports
                .insert((ident.clone(), *location));
        }
    }

    let crate_files = context.crate_files(&crate_id);
    for error in frontend_errors {
        if let FrontendError::ResolverError(ResolverError::VariableDoesNotNeedToBeMutable {
            ident,
        }) = error
        {
            let location = ident.location();
            if crate_files.contains(&location.file) {
                fixes_per_file.entry(location.file).or_default().unnecessary_muts.insert(location);
            }
        }
    }

    // Sort by path so files are reported in a deterministic order.
    let mut file_ids: Vec<FileId> = fixes_per_file.keys().copied().collect();
    file_ids.sort_by_key(|file_id| context.file_manager.path(*file_id).map(Path::to_path_buf));

    let mut fixed_locations = HashSet::new();
    for file_id in file_ids {
        let fixes = &fixes_per_file[&file_id];
        let Some((parsed_module, _)) = parsed_files.get(&file_id) else {
            continue;
        };
        let Some(source) = context.file_manager.fetch_file(file_id) else {
            continue;
        };
        let Some(new_source) = apply_fixes(source, parsed_module, fixes) else {
            continue;
        };
        let Some(path) = context.file_manager.path(file_id) else {
            continue;
        };
        std::fs::write(path, new_source).map_err(|error| {
            CliError::Generic(format!("Failed to write {}: {error}", path.display()))
        })?;
        println!("Fixed {}", path.display());
        fixed_locations.extend(fixes.unused_imports.iter().map(|(_ident, location)| *location));
        fixed_locations.extend(fixes.unnecessary_muts.iter().copied());
    }

    Ok(fixed_locations)
}

/// Removes from `warnings` the ones that were just fixed: the ones whose label points at code
/// that was rewritten (a removed import, a dropped `mut`). Other diagnostics (warnings at
/// other locations, and anything that is not a warning) are left untouched.
fn remove_fixed_warnings(
    warnings: &mut Vec<CustomDiagnostic>,
    fixed_locations: &HashSet<Location>,
) {
    warnings.retain(|diagnostic| {
        !(diagnostic.is_warning()
            && diagnostic.secondaries.iter().any(|label| fixed_locations.contains(&label.location)))
    });
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
                let default_value_vec =
                    std::iter::repeat_n(default_value(*typ), length.try_into().unwrap()).collect();
                toml::Value::Array(default_value_vec)
            }
            AbiType::Struct { fields, .. } => {
                let default_value_map = toml::map::Map::from_iter(
                    fields.into_iter().map(|(name, typ)| (name, default_value(typ))),
                );
                toml::Value::Table(default_value_map)
            }
            AbiType::Field | AbiType::Integer { .. } => toml::Value::Integer(0),
            AbiType::Boolean => toml::Value::Boolean(false),
            AbiType::Tuple { fields } => {
                let default_value_vec = fields.into_iter().map(default_value).collect();
                toml::Value::Array(default_value_vec)
            }
            AbiType::String { length } => toml::Value::String("_".repeat(length as usize)),
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
    use std::collections::HashSet;

    use fm::FileId;
    use insta::assert_snapshot;
    use noirc_abi::{AbiParameter, AbiType, AbiVisibility, Sign};
    use noirc_errors::{CustomDiagnostic, Location, Span};

    use super::{create_input_toml_template, remove_fixed_warnings};

    #[test]
    fn removes_only_warnings_at_removed_import_locations() {
        let file = FileId::dummy();
        let removed_location = Location::new(Span::from(10..13), file);
        let other_location = Location::new(Span::from(20..23), file);

        let fixed_warning = CustomDiagnostic::simple_warning(
            "unused import foo".to_string(),
            "unused import".to_string(),
            removed_location,
        );
        let unrelated_warning = CustomDiagnostic::simple_warning(
            "unused variable x".to_string(),
            "unused variable".to_string(),
            other_location,
        );
        // Not a warning, so it must survive even though it points at a removed location.
        let error_at_removed_location = CustomDiagnostic::simple_error(
            "some error".to_string(),
            "error".to_string(),
            removed_location,
        );

        let mut warnings = vec![fixed_warning, unrelated_warning, error_at_removed_location];
        let removed_import_locations = HashSet::from([removed_location]);
        remove_fixed_warnings(&mut warnings, &removed_import_locations);

        let messages: Vec<&str> =
            warnings.iter().map(|diagnostic| diagnostic.message.as_str()).collect();
        assert_eq!(messages, vec!["unused variable x", "some error"]);
    }

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
            typed_param(
                "f",
                AbiType::Tuple { fields: vec![AbiType::Field, AbiType::String { length: 5 }] },
            ),
        ];

        let return_type = AbiType::Boolean;

        let toml_str = create_input_toml_template(parameters, Some(return_type));

        assert_snapshot!(toml_str, @r#"
        a = 0
        b = 0
        c = [0, 0]
        e = false
        f = [0, "_____"]
        return = false

        [d]
        d1 = 0
        d2 = [0, 0, 0]
        "#);
    }
}
