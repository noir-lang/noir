use clap::Args;
use fm::FileManager;
use nargo::{
    errors::CompileError, insert_all_files_for_workspace_into_file_manager, package::Package,
    parse_all, prepare_package, workspace::Workspace,
};
use nargo_toml::PackageSelection;
use noirc_driver::CompileOptions;
use noirc_frontend::hir::{
    Context, ParsedFiles,
    def_map::{CrateDefMap, ModuleId},
};
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
        check_package(&workspace_file_manager, &parsed_files, package, &args.compile_options)?;
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
    check_crate_and_report_errors(&mut context, crate_id, compile_options)?;

    let def_map = &context.def_maps[&crate_id];
    let root_module_id = def_map.root();
    let module_id = ModuleId { krate: crate_id, local_id: root_module_id };

    let mut string = String::new();
    show_module(module_id, &context, def_map, &mut string);
    println!("{}", string);

    Ok(())
}

fn show_module(module_id: ModuleId, context: &Context, def_map: &CrateDefMap, string: &mut String) {
    let attributes = context.def_interner.try_module_attributes(&module_id);
    let name =
        attributes.map(|attributes| attributes.name.clone()).unwrap_or_else(|| String::new());

    let module_data = &def_map.modules()[module_id.local_id.0];
    let definitions = module_data.definitions();

    for (name, scope) in definitions.types().iter().chain(definitions.values()) {
        for (_trait_id, (module_def_id, visibility, _is_prelude)) in scope {
            let mut printer = Printer::new(&context.def_interner, string);
            printer.show_module_def_id(*module_def_id, *visibility);
        }
    }
}
