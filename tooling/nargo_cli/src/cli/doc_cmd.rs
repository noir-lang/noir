use clap::Args;
use fm::FileManager;
use nargo::{
    errors::CompileError, insert_all_files_for_workspace_into_file_manager,
    ops::check_crate_and_report_errors, package::Package, parse_all, prepare_package,
    workspace::Workspace,
};
use nargo_doc::{
    crate_to_item,
    items::{Crate, Crates, Item},
};
use nargo_toml::PackageSelection;
use noirc_driver::CompileOptions;
use noirc_frontend::hir::ParsedFiles;

use crate::errors::CliError;

use super::{LockType, PackageOptions, WorkspaceCommand};

#[derive(Debug, Clone, Args, Default)]
pub(crate) struct DocCommand {
    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    compile_options: CompileOptions,
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
    for package in &workspace {
        let item =
            package_item(&workspace_file_manager, &parsed_files, package, &args.compile_options)?;
        crates.push(Crate { name: package.name.to_string(), items: vec![item] });
    }
    let crates = Crates { crates };

    match serde_json::to_string_pretty(&crates) {
        Ok(json) => {
            println!("{json}");
            Ok(())
        }
        Err(_) => todo!("Handle error case"),
    }
}

fn package_item(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    compile_options: &CompileOptions,
) -> Result<Item, CompileError> {
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);

    check_crate_and_report_errors(&mut context, crate_id, compile_options)?;

    Ok(crate_to_item(crate_id, &context.crate_graph, &context.def_maps, &context.def_interner))
}
