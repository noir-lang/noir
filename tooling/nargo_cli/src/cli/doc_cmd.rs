use std::{collections::HashMap, fs};

use clap::Args;
use fm::FileManager;
use nargo::{
    errors::CompileError, insert_all_files_for_workspace_into_file_manager,
    ops::check_crate_and_report_errors, package::Package, parse_all, prepare_package,
    workspace::Workspace,
};
use nargo_doc::{
    ItemIds, crate_module,
    items::{Crate, Crates, Module},
};
use nargo_toml::PackageSelection;
use noirc_driver::CompileOptions;
use noirc_frontend::hir::ParsedFiles;

use crate::errors::CliError;

use super::{LockType, PackageOptions, WorkspaceCommand};

#[derive(Debug, Clone, Args, Default)]
pub(crate) struct DocCommand {
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
    let mut ids = HashMap::new();
    for package in &workspace {
        let root_module = package_module(
            &workspace_file_manager,
            &parsed_files,
            package,
            &args.compile_options,
            &mut ids,
        )?;
        crates.push(Crate { name: package.name.to_string(), root_module });
    }
    let name = workspace.root_dir.file_name().unwrap().to_string_lossy().to_string();
    let crates = Crates { crates, name };

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

fn package_module(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    compile_options: &CompileOptions,
    ids: &mut ItemIds,
) -> Result<Module, CompileError> {
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);

    check_crate_and_report_errors(&mut context, crate_id, compile_options)?;

    Ok(crate_module(crate_id, &context.crate_graph, &context.def_maps, &context.def_interner, ids))
}
