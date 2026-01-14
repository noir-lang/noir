use std::{collections::HashMap, fs};

use clap::Args;
use fm::FileManager;
use iter_extended::vecmap;
use nargo::{
    errors::CompileError, insert_all_files_for_workspace_into_file_manager,
    ops::check_crate_and_report_errors, package::Package, parse_all, prepare_package,
    workspace::Workspace,
};
use nargo_doc::{BrokenLink, crate_module, items::Crate};
use nargo_toml::{PackageConfig, PackageSelection};
use noirc_driver::{CompileOptions, CrateId, stdlib_nargo_toml_source};
use noirc_errors::CustomDiagnostic;
use noirc_frontend::hir::{Context, ParsedFiles};

use crate::errors::CliError;

use super::{LockType, PackageOptions, WorkspaceCommand};

/// Builds documentation for the specified package or workspace.
///
/// Note: this command is in development and functionality may change greatly with no warning.
#[derive(Debug, Clone, Args, Default)]
pub(crate) struct DocCommand {
    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    compile_options: CompileOptions,

    /// Do not produce any output files, only check for broken links.
    #[clap(long)]
    check: bool,
}

impl WorkspaceCommand for DocCommand {
    fn package_selection(&self) -> PackageSelection {
        self.package_options.package_selection()
    }
    fn lock_type(&self) -> LockType {
        // Creates a `Prover.toml` template if it doesn't exist, otherwise only writes if `allow_overwrite` is true,
        // so it shouldn't lead to accidental conflicts. Doesn't produce compilation artifacts.
        LockType::None
    }
}

pub(crate) fn run(args: DocCommand, workspace: Workspace) -> Result<(), CliError> {
    let mut workspace_file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = parse_all(&workspace_file_manager);

    let mut broken_links = Vec::new();

    // Maps a crate's root file to its crate
    let mut dependencies = HashMap::new();

    let mut crates = Vec::new();
    for package in &workspace {
        let krate = package_crate(
            &workspace_file_manager,
            &parsed_files,
            package,
            &args.compile_options,
            &mut dependencies,
            &mut broken_links,
        )?;
        crates.push(krate);
    }

    // Report broken links
    let diagnostics = vecmap(&broken_links, CustomDiagnostic::from);
    let deny_warnings = args.compile_options.deny_warnings || args.check;
    noirc_errors::reporter::report_all(
        workspace_file_manager.as_file_map(),
        &diagnostics,
        deny_warnings,
        args.compile_options.silence_warnings,
    );

    if args.check {
        if !broken_links.is_empty() {
            let msg = if broken_links.len() == 1 {
                "Error: doc comments contains 1 broken link".to_string()
            } else {
                format!("Error: doc comments contain {} broken links", broken_links.len())
            };
            return Err(CliError::Generic(msg));
        }
        return Ok(());
    }

    // Crates in the workspace might depend on other crates in the workspace.
    // Remove them from `all_dependencies`.
    for krate in &crates {
        dependencies.remove(&krate.root_file);
    }
    let dependencies = dependencies.into_values().collect::<Vec<_>>();

    let name = workspace.root_dir.file_name().unwrap().to_string_lossy().to_string();
    let crates = nargo_doc::items::Workspace { crates, name, dependencies };

    let files = nargo_doc::to_html(&crates);
    let target_dir = workspace.target_directory_path();
    let docs_dir = target_dir.join("docs");
    if let Ok(true) = fs::exists(&docs_dir) {
        fs::remove_dir_all(&docs_dir).map_err(|err| {
            CliError::Generic(format!("Failed to remove existing docs directory: {err}"))
        })?;
    }
    for (path, contents) in files {
        let full_path = docs_dir.join(path);
        fs::create_dir_all(full_path.parent().unwrap())
            .map_err(|err| CliError::Generic(format!("Failed to create directory: {err}")))?;
        fs::write(full_path, contents)
            .map_err(|err| CliError::Generic(format!("Failed to write file: {err}")))?;
    }
    println!("Generated {}", docs_dir.join("index.html").display());
    Ok(())
}

/// Returns the Crate item for the given package, together with all of
/// its dependencies in a HashMap that maps a dependency's root file to
/// its crate.
fn package_crate(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    compile_options: &CompileOptions,
    dependencies: &mut HashMap<String, Crate>,
    broken_links: &mut Vec<BrokenLink>,
) -> Result<Crate, CompileError> {
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);

    check_crate_and_report_errors(&mut context, crate_id, compile_options)?;

    let (module, module_broken_links) = crate_module(
        crate_id,
        &context.crate_graph,
        &context.def_maps,
        &context.def_interner,
        file_manager,
    );
    broken_links.extend(module_broken_links);

    collect_dependencies(
        &context,
        Some(package),
        crate_id,
        file_manager,
        dependencies,
        broken_links,
    )?;

    let root_file = &context.crate_graph[crate_id].root_file_id;
    let root_file = file_manager.path(*root_file).unwrap().display().to_string();

    let name = package.name.to_string();
    let version = package.version.clone();
    let krate = Crate { name, version, root_module: module, root_file };
    Ok(krate)
}

fn collect_dependencies(
    context: &Context,
    package: Option<&Package>,
    crate_id: CrateId,
    file_manager: &FileManager,
    dependencies: &mut HashMap<String, Crate>,
    broken_links: &mut Vec<BrokenLink>,
) -> Result<(), CompileError> {
    for dependency in &context.crate_graph[crate_id].dependencies {
        let crate_id = dependency.crate_id;
        let crate_data = &context.crate_graph[crate_id];
        let root_file = crate_data.root_file_id;
        let root_file = file_manager.path(root_file).unwrap().display().to_string();
        if dependencies.contains_key(&root_file) {
            continue;
        }

        let (module, module_broken_links) = crate_module(
            crate_id,
            &context.crate_graph,
            &context.def_maps,
            &context.def_interner,
            file_manager,
        );
        broken_links.extend(module_broken_links);

        let name = dependency.name.to_string();
        // The `graph::Dependency` type doesn't carry a version. Instead, we can get it from the
        // `Package's` dependencies by finding the dependency with the same name.
        // This doesn't work for the standard library, because it's never loaded as a `Package`.
        // In that case we get the version in a different way.
        let package_dependency = package
            .and_then(|package| {
                package.dependencies.iter().find(|(_crate_name, package_dependency)| {
                    package_dependency.package_name() == &dependency.name
                })
            })
            .map(|(_, dependency)| dependency);
        let package = package_dependency.map(|dependency| dependency.package());
        let mut version = package.and_then(|package| package.version.clone());

        if crate_id.is_stdlib() {
            let stdlib_toml: PackageConfig = toml::from_str(&stdlib_nargo_toml_source()).unwrap();
            version = stdlib_toml.package.version.clone();
        }

        let krate = Crate { name, version, root_module: module, root_file: root_file.clone() };
        dependencies.insert(root_file, krate);

        collect_dependencies(context, package, crate_id, file_manager, dependencies, broken_links)?;
    }
    Ok(())
}
