use clap::Args;
use fm::FileManager;
use items::ItemBuilder;
use nargo::{
    errors::CompileError, insert_all_files_for_workspace_into_file_manager,
    ops::check_crate_and_report_errors, package::Package, parse_all, prepare_package,
    workspace::Workspace,
};
use nargo_fmt::ImportsGranularity;
use nargo_toml::PackageSelection;
use noirc_driver::CompileOptions;
use noirc_frontend::{
    hir::{ParsedFiles, def_map::ModuleId},
    parse_program_with_dummy_file,
};
use printer::ItemPrinter;

use crate::errors::CliError;

use super::{LockType, PackageOptions, WorkspaceCommand};

mod items;
mod printer;

/// Show the result of macro expansion
#[derive(Debug, Clone, Args, Default)]
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
    let code = get_expanded_package(file_manager, parsed_files, package, compile_options)?;
    println!("{code}");
    Ok(())
}

fn get_expanded_package(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    compile_options: &CompileOptions,
) -> Result<String, CompileError> {
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);

    // Even though this isn't LSP, we need to active this to be able to go from a ModuleDefId to its parent module
    context.activate_lsp_mode();

    check_crate_and_report_errors(&mut context, crate_id, compile_options)?;

    let root_module_id = context.def_maps[&crate_id].root();
    let module_id = ModuleId { krate: crate_id, local_id: root_module_id };

    let mut builder = ItemBuilder::new(crate_id, &context.def_interner, &context.def_maps);
    let item = builder.build_module(module_id);

    let dependencies = &context.crate_graph[context.root_crate_id()].dependencies;

    let mut string = String::new();
    let mut printer = ItemPrinter::new(
        crate_id,
        &context.def_interner,
        &context.def_maps,
        dependencies,
        &mut string,
    );
    printer.show_item(item);

    let (parsed_module, errors) = parse_program_with_dummy_file(&string);
    if errors.is_empty() {
        let config = nargo_fmt::Config {
            reorder_imports: true,
            imports_granularity: ImportsGranularity::Crate,
            ..Default::default()
        };
        Ok(nargo_fmt::format(&string, parsed_module, &config))
    } else {
        string.push_str("\n\n// Warning: the generated code has syntax errors");
        Ok(string)
    }
}
