use clap::Args;
use fm::FileManager;
use nargo::{
    errors::CompileError, insert_all_files_for_workspace_into_file_manager, package::Package,
    parse_all, prepare_package, workspace::Workspace,
};
use nargo_toml::PackageSelection;
use noirc_driver::CompileOptions;
use noirc_frontend::hir::{ParsedFiles, def_map::ModuleId};
use printer::Printer;

use crate::errors::CliError;

use super::{LockType, PackageOptions, WorkspaceCommand, check_cmd::check_crate_and_report_errors};

mod printer;

/// Expands macros
#[derive(Debug, Clone, Args)]
pub(crate) struct ExpandCommand {
    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

impl WorkspaceCommand for ExpandCommand {
    fn package_selection(&self) -> PackageSelection {
        self.package_options.package_selection()
    }
    fn lock_type(&self) -> LockType {
        // Creates a `Prover.toml` template if it doesn't exist, otherwise only writes if `allow_overwrite` is true,
        // so it shouldn't lead to accidental conflicts. Doesn't produce compilation artifacts.
        LockType::None
    }
}

pub(crate) fn run(args: ExpandCommand, workspace: Workspace) -> Result<(), CliError> {
    let mut workspace_file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = parse_all(&workspace_file_manager);

    for package in &workspace {
        expand_package(&workspace_file_manager, &parsed_files, package, &args.compile_options)?;
    }

    Ok(())
}

fn expand_package(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    compile_options: &CompileOptions,
) -> Result<(), CompileError> {
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);
    check_crate_and_report_errors(&mut context, crate_id, compile_options)?;

    let def_map = &context.def_maps[&crate_id];
    let root_module_id = def_map.root();
    let module_id = ModuleId { krate: crate_id, local_id: root_module_id };

    let mut string = String::new();
    let mut printer = Printer::new(crate_id, &context.def_interner, def_map, &mut string);
    printer.show_module(module_id);
    printer.show_stray_trait_impls();
    println!("{}", string);

    Ok(())
}
