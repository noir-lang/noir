use std::{collections::HashMap, fs};

use clap::Args;
use fm::FileManager;
use nargo::{
    errors::CompileError, insert_all_files_for_workspace_into_file_manager,
    ops::check_crate_and_report_errors, package::Package, parse_all, prepare_package,
    workspace::Workspace,
};
use nargo_doc::{ItemIds, crate_module, items::Crate};
use nargo_toml::PackageSelection;
use noirc_driver::{CompileOptions, CrateId};
use noirc_frontend::hir::{Context, ParsedFiles};

use crate::errors::CliError;

use super::{LockType, PackageOptions, WorkspaceCommand};

/// Builds documentation for the specified package or workspace.
#[derive(Debug, Clone, Args, Default)]
pub(crate) struct DocCommand {
    /// Output format for the documentation, HTML by default.
    #[clap(long)]
    format: Option<Format>,

    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum Format {
    Json,
    Html,
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

    let mut crates = Vec::new();
    // Maps a crate's root file to its crate
    let mut all_dependencies = HashMap::new();
    let mut ids = HashMap::new();
    for package in &workspace {
        let (krate, dependencies) = package_crate(
            &workspace_file_manager,
            &parsed_files,
            package,
            &args.compile_options,
            &mut ids,
        )?;
        crates.push(krate);
        all_dependencies.extend(dependencies);
    }
    // Crates in the workspace might depend on other crates in the workspace.
    // Remove them from `all_dependencies`.
    for krate in &crates {
        all_dependencies.remove(&krate.root_file);
    }
    let dependencies = all_dependencies.into_values().collect::<Vec<_>>();

    let name = workspace.root_dir.file_name().unwrap().to_string_lossy().to_string();
    let crates = nargo_doc::items::Workspace { crates, name, dependencies };

    let format = args.format.unwrap_or(Format::Html);
    match format {
        Format::Json => match serde_json::to_string_pretty(&crates) {
            Ok(json) => {
                println!("{json}");
                Ok(())
            }
            Err(err) => Err(CliError::Generic(err.to_string())),
        },
        Format::Html => {
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
                fs::create_dir_all(full_path.parent().unwrap()).map_err(|err| {
                    CliError::Generic(format!("Failed to create directory: {err}"))
                })?;
                fs::write(full_path, contents)
                    .map_err(|err| CliError::Generic(format!("Failed to write file: {err}")))?;
            }
            println!("Generated {}", docs_dir.join("index.html").display());
            Ok(())
        }
    }
}

/// Returns the Crate item for the given package, together with all of
/// its dependencies in a HashMap that maps a dependency's root file to
/// its crate.
fn package_crate(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    compile_options: &CompileOptions,
    ids: &mut ItemIds,
) -> Result<(Crate, HashMap<String, Crate>), CompileError> {
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);

    check_crate_and_report_errors(&mut context, crate_id, compile_options)?;

    let module =
        crate_module(crate_id, &context.crate_graph, &context.def_maps, &context.def_interner, ids);

    let mut dependencies = HashMap::new();
    collect_dependencies(&context, crate_id, file_manager, &mut dependencies, ids)?;

    let root_file = &context.crate_graph[crate_id].root_file_id;
    let root_file = file_manager.path(*root_file).unwrap().display().to_string();

    let krate = Crate { name: package.name.to_string(), root_module: module, root_file };
    Ok((krate, dependencies))
}

fn collect_dependencies(
    context: &Context,
    crate_id: CrateId,
    file_manager: &FileManager,
    dependencies: &mut HashMap<String, Crate>,
    ids: &mut ItemIds,
) -> Result<(), CompileError> {
    for dependency in &context.crate_graph[crate_id].dependencies {
        let crate_id = dependency.crate_id;
        let root_file = &context.crate_graph[crate_id].root_file_id;
        let root_file = file_manager.path(*root_file).unwrap().display().to_string();
        let module = crate_module(
            crate_id,
            &context.crate_graph,
            &context.def_maps,
            &context.def_interner,
            ids,
        );
        let name = dependency.name.to_string();
        dependencies.insert(root_file.clone(), Crate { name, root_module: module, root_file });

        collect_dependencies(context, crate_id, file_manager, dependencies, ids)?;
    }
    Ok(())
}
