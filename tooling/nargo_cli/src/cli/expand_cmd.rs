use clap::Args;
use fm::FileManager;
use items::ItemBuilder;
use nargo::{
    errors::CompileError, insert_all_files_for_workspace_into_file_manager, package::Package,
    parse_all, prepare_package, workspace::Workspace,
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

use super::{LockType, PackageOptions, WorkspaceCommand, check_cmd::check_crate_and_report_errors};

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

    let mut string = String::new();
    let mut printer =
        ItemPrinter::new(crate_id, &context.def_interner, &context.def_maps, &mut string);
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

#[cfg(test)]
mod tests {
    use nargo::{insert_all_files_for_workspace_into_file_manager, parse_all};
    use nargo_toml::PackageSelection;
    use noirc_frontend::elaborator::UnstableFeature;

    use crate::cli::read_workspace;

    use super::{ExpandCommand, get_expanded_package};

    #[test]
    fn nargo_expand_smoke_test() {
        let root_path = std::env::current_dir()
            .unwrap()
            .join("test_programs")
            .join("expand")
            .canonicalize()
            .expect("Could not resolve root path");

        let mut command = ExpandCommand::default();
        command.compile_options.unstable_features.push(UnstableFeature::Enums);
        let workspace = read_workspace(&root_path, PackageSelection::All).unwrap();

        let mut workspace_file_manager = workspace.new_file_manager();
        insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
        let parsed_files = parse_all(&workspace_file_manager);

        let code = get_expanded_package(
            &workspace_file_manager,
            &parsed_files,
            workspace.into_iter().next().unwrap(),
            &command.compile_options,
        )
        .expect("Expected code to be expanded");

        let expected = r#"use std::as_witness as aliased_as_witness;

trait SomeTrait {
    /// some_method docs
    fn some_method();
}

/// Docs on top of Foo
pub struct Foo<T> {
    /// Docs on top of value
    value: T,
    int: i32,
}

impl<T> Foo<T> {
    fn method(self, x: i32) -> i32 {
        self.int() + x
    }

    /// int docs
    pub fn int(self) -> i32 {
        self.int
    }
}

impl<T> SomeTrait for Foo<T> {
    /// some_method docs for Foo
    fn some_method() {
        let name: str<11> = "some_method";
        panic(f"Implement \\\n\t {name} {{ }}")
    }
}

mod module {
    pub struct Bar {}
}

fn main() {
    let _x: () = aliased_as_witness(1);
    let _y: Field = 3;
    let foo: Foo<Field> = Foo::<Field> { value: 1, int: 2 };
    let _: i32 = foo.method(10);
}

#[abi(functions)]
fn bar() -> module::Bar {
    let bar: module::Bar = module::Bar {};
    bar
}

comptime fn generate_baz(_: Module) -> Quoted {
    quote {
        pub fn baz() {
            
        }
    }
}

pub fn baz() {}

fn test_enums() {
    let foo: Foo<Field> = Foo::<Field> { value: 1, int: 2 };
    {
        let internal___variable: Foo<Field> = foo;
        match internal___variable {
            Foo::<Field> { value: internal_match_variable_0, int: internal_match_variable_1 } => {
                let value: Field = internal_match_variable_1;
                {
                    let int: i32 = internal_match_variable_0;
                    {
                        println(f"value: {value}");
                        println(f"int: {int}");
                    }
                }
            },
        }
    }
}
"#;

        assert_eq!(code, expected);
    }
}
