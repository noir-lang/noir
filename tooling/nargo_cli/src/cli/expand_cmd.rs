use clap::Args;
use fm::FileManager;
use nargo::{
    errors::CompileError, insert_all_files_for_workspace_into_file_manager, package::Package,
    parse_all, prepare_package, workspace::Workspace,
};
use nargo_toml::PackageSelection;
use noirc_driver::CompileOptions;
use noirc_frontend::{
    Generics, Type,
    ast::{ItemVisibility, Visibility},
    hir::{
        Context, ParsedFiles,
        def_map::{CrateDefMap, ModuleDefId, ModuleId},
    },
    hir_def::{expr::HirExpression, stmt::HirPattern},
    node_interner::{FuncId, NodeInterner},
};

use crate::errors::CliError;

use super::{LockType, PackageOptions, WorkspaceCommand, check_cmd::check_crate_and_report_errors};

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
            show_module_def_id(*module_def_id, *visibility, &context.def_interner, string);
        }
    }
}

fn show_module_def_id(
    module_def_id: ModuleDefId,
    visibility: ItemVisibility,
    interner: &NodeInterner,
    string: &mut String,
) {
    if visibility != ItemVisibility::Private {
        string.push_str(&visibility.to_string());
        string.push(' ');
    };

    match module_def_id {
        ModuleDefId::ModuleId(module_id) => todo!("Show modules"),
        ModuleDefId::FunctionId(func_id) => show_function(func_id, interner, string),
        ModuleDefId::TypeId(_) => todo!("Show types"),
        ModuleDefId::TypeAliasId(type_alias_id) => todo!("Show type aliases"),
        ModuleDefId::TraitId(trait_id) => todo!("Show traits"),
        ModuleDefId::GlobalId(global_id) => todo!("Show globals"),
    }

    string.push_str("\n\n");
}

fn show_function(func_id: FuncId, interner: &NodeInterner, string: &mut String) {
    let modifiers = interner.function_modifiers(&func_id);
    let func_meta = interner.function_meta(&func_id);
    let name = &modifiers.name;

    if modifiers.is_unconstrained {
        string.push_str("unconstrained ");
    }
    if modifiers.is_comptime {
        string.push_str("comptime ");
    }

    string.push_str("fn ");
    string.push_str(name);

    show_generics(&func_meta.direct_generics, string);

    string.push('(');
    let parameters = &func_meta.parameters;
    for (index, (pattern, typ, visibility)) in parameters.iter().enumerate() {
        let is_self = pattern_is_self(pattern, interner);

        // `&mut self` is represented as a mutable reference type, not as a mutable pattern
        if is_self && matches!(typ, Type::Reference(..)) {
            string.push_str("&mut ");
        }

        show_pattern(pattern, interner, string);

        // Don't add type for `self` param
        if !is_self {
            string.push_str(": ");
            if matches!(visibility, Visibility::Public) {
                string.push_str("pub ");
            }
            string.push_str(&format!("{}", typ));
        }

        if index != parameters.len() - 1 {
            string.push_str(", ");
        }
    }
    string.push(')');

    let return_type = func_meta.return_type();
    match return_type {
        Type::Unit => (),
        _ => {
            string.push_str(" -> ");
            string.push_str(&format!("{}", return_type));
        }
    }

    string.push(' ');

    let hir_function = interner.function(&func_id);
    let block = hir_function.block(interner);
    let block = HirExpression::Block(block);
    let block = block.to_display_ast(interner, func_meta.location);
    string.push_str(&block.to_string());
}

fn show_generics(generics: &Generics, string: &mut String) {
    show_generics_impl(
        generics, false, // only show names
        string,
    );
}

fn show_generic_names(generics: &Generics, string: &mut String) {
    show_generics_impl(
        generics, true, // only show names
        string,
    );
}

fn show_generics_impl(generics: &Generics, only_show_names: bool, string: &mut String) {
    if generics.is_empty() {
        return;
    }

    string.push('<');
    for (index, generic) in generics.iter().enumerate() {
        if index > 0 {
            string.push_str(", ");
        }

        if only_show_names {
            string.push_str(&generic.name);
        } else {
            match generic.kind() {
                noirc_frontend::Kind::Any | noirc_frontend::Kind::Normal => {
                    string.push_str(&generic.name);
                }
                noirc_frontend::Kind::IntegerOrField | noirc_frontend::Kind::Integer => {
                    string.push_str("let ");
                    string.push_str(&generic.name);
                    string.push_str(": u32");
                }
                noirc_frontend::Kind::Numeric(typ) => {
                    string.push_str("let ");
                    string.push_str(&generic.name);
                    string.push_str(": ");
                    string.push_str(&typ.to_string());
                }
            }
        }
    }
    string.push('>');
}

fn show_pattern(pattern: &HirPattern, interner: &NodeInterner, string: &mut String) {
    match pattern {
        HirPattern::Identifier(ident) => {
            let definition = interner.definition(ident.id);
            string.push_str(&definition.name);
        }
        HirPattern::Mutable(pattern, _) => {
            string.push_str("mut ");
            show_pattern(pattern, interner, string);
        }
        HirPattern::Tuple(..) | HirPattern::Struct(..) => {
            string.push('_');
        }
    }
}

fn pattern_is_self(pattern: &HirPattern, interner: &NodeInterner) -> bool {
    match pattern {
        HirPattern::Identifier(ident) => {
            let definition = interner.definition(ident.id);
            definition.name == "self"
        }
        HirPattern::Mutable(pattern, _) => pattern_is_self(pattern, interner),
        HirPattern::Tuple(..) | HirPattern::Struct(..) => false,
    }
}
