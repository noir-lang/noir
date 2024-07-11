use std::future::{self, Future};

use async_lsp::ResponseError;
use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind};
use noirc_frontend::{
    ast::Visibility,
    graph::CrateId,
    hir::def_map::ModuleId,
    hir_def::stmt::HirPattern,
    macros_api::{NodeInterner, StructId},
    node_interner::{
        DefinitionId, DefinitionKind, FuncId, GlobalId, ReferenceId, TraitId, TypeAliasId,
    },
    Generics, Type,
};

use crate::LspState;

use super::{process_request, to_lsp_location, ProcessRequestCallbackArgs};

pub(crate) fn on_hover_request(
    state: &mut LspState,
    params: HoverParams,
) -> impl Future<Output = Result<Option<Hover>, ResponseError>> {
    let result = process_request(state, params.text_document_position_params, |args| {
        args.interner.reference_at_location(args.location).map(|reference| {
            let location = args.interner.reference_location(reference);
            let lsp_location = to_lsp_location(args.files, location.file, location.span);
            Hover {
                range: lsp_location.map(|location| location.range),
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format_reference(reference, &args),
                }),
            }
        })
    });

    future::ready(result)
}

fn format_reference(reference: ReferenceId, args: &ProcessRequestCallbackArgs) -> String {
    match reference {
        ReferenceId::Module(id) => format_module(id, args),
        ReferenceId::Struct(id) => format_struct(id, args),
        ReferenceId::StructMember(id, field_index) => format_struct_member(id, field_index, args),
        ReferenceId::Trait(id) => format_trait(id, args),
        ReferenceId::Global(id) => format_global(id, args),
        ReferenceId::Function(id) => format_function(id, args),
        ReferenceId::Alias(id) => format_alias(id, args),
        ReferenceId::Local(id) => format_local(id, &args),
        ReferenceId::Reference(location, _) => {
            format_reference(args.interner.find_referenced(location).unwrap(), args)
        }
    }
}
fn format_module(id: ModuleId, args: &ProcessRequestCallbackArgs) -> String {
    let module_attributes = args.interner.module_attributes(&id);

    let mut string = String::new();
    if format_parent_module_from_module_id(
        &ModuleId { krate: id.krate, local_id: module_attributes.parent },
        args,
        &mut string,
    ) {
        string.push_str("\n");
    }
    string.push_str("    ");
    string.push_str("mod ");
    string.push_str(&module_attributes.name);
    string
}

fn format_struct(id: StructId, args: &ProcessRequestCallbackArgs) -> String {
    let struct_type = args.interner.get_struct(id);
    let struct_type = struct_type.borrow();

    let mut string = String::new();
    if format_parent_module(ReferenceId::Struct(id), args, &mut string) {
        string.push_str("\n");
    }
    string.push_str("    ");
    string.push_str("struct ");
    string.push_str(&struct_type.name.0.contents);
    format_generics(&struct_type.generics, &mut string);
    string.push_str(" {\n");
    for (field_name, field_type) in struct_type.get_fields_as_written() {
        string.push_str("        ");
        string.push_str(&field_name);
        string.push_str(": ");
        string.push_str(&format!("{}", field_type));
        string.push_str(",\n");
    }
    string.push_str("    }");
    string
}

fn format_struct_member(
    id: StructId,
    field_index: usize,
    args: &ProcessRequestCallbackArgs,
) -> String {
    let struct_type = args.interner.get_struct(id);
    let struct_type = struct_type.borrow();
    let (field_name, field_type) = struct_type.field_at(field_index);

    let mut string = String::new();
    if format_parent_module(ReferenceId::Struct(id), args, &mut string) {
        string.push_str("::");
    }
    string.push_str(&struct_type.name.0.contents);
    string.push_str("\n");
    string.push_str("    ");
    string.push_str(&field_name.0.contents);
    string.push_str(": ");
    string.push_str(&format!("{}", field_type));
    string
}

fn format_trait(id: TraitId, args: &ProcessRequestCallbackArgs) -> String {
    let a_trait = args.interner.get_trait(id);

    let mut string = String::new();
    if format_parent_module(ReferenceId::Trait(id), args, &mut string) {
        string.push_str("\n");
    }
    string.push_str("    ");
    string.push_str("trait ");
    string.push_str(&a_trait.name.0.contents);
    format_generics(&a_trait.generics, &mut string);
    string
}

fn format_global(id: GlobalId, args: &ProcessRequestCallbackArgs) -> String {
    let global_info = args.interner.get_global(id);
    let definition_id = global_info.definition_id;
    let typ = args.interner.definition_type(definition_id);

    let mut string = String::new();
    if format_parent_module(ReferenceId::Global(id), args, &mut string) {
        string.push_str("\n");
    }
    string.push_str("    ");
    string.push_str("global ");
    string.push_str(&global_info.ident.0.contents);
    string.push_str(": ");
    string.push_str(&format!("{}", typ));
    string
}

fn format_function(id: FuncId, args: &ProcessRequestCallbackArgs) -> String {
    let func_meta = args.interner.function_meta(&id);
    let func_name_definition_id = args.interner.definition(func_meta.name.id);

    let mut string = String::new();
    let formatted_parent_module =
        format_parent_module(ReferenceId::Function(id), args, &mut string);
    let formatted_parent_struct = if let Some(struct_id) = func_meta.struct_id {
        let struct_type = args.interner.get_struct(struct_id);
        let struct_type = struct_type.borrow();
        if formatted_parent_module {
            string.push_str("::");
        }
        string.push_str(&struct_type.name.0.contents);
        true
    } else {
        false
    };
    if formatted_parent_module || formatted_parent_struct {
        string.push_str("\n");
    }
    string.push_str("    ");
    string.push_str("fn ");
    format_generics(&func_meta.direct_generics, &mut string);
    string.push_str(&func_name_definition_id.name);
    string.push('(');
    let parameters = &func_meta.parameters;
    for (index, (pattern, typ, visibility)) in parameters.iter().enumerate() {
        format_pattern(pattern, args.interner, &mut string);
        string.push_str(": ");
        if matches!(visibility, Visibility::Public) {
            string.push_str("pub ");
        }
        string.push_str(&format!("{}", typ));
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

    string
}

fn format_alias(id: TypeAliasId, args: &ProcessRequestCallbackArgs) -> String {
    let type_alias = args.interner.get_type_alias(id);
    let type_alias = type_alias.borrow();

    let mut string = String::new();
    format_parent_module(ReferenceId::Alias(id), args, &mut string);
    string.push_str("\n");
    string.push_str("    ");
    string.push_str("type ");
    string.push_str(&type_alias.name.0.contents);
    string.push_str(" = ");
    string.push_str(&format!("{}", &type_alias.typ));
    string
}

fn format_local(id: DefinitionId, args: &ProcessRequestCallbackArgs) -> String {
    let definition_info = args.interner.definition(id);
    let DefinitionKind::Local(expr_id) = definition_info.kind else {
        panic!("Expected a local reference to reference a local definition")
    };
    let typ = args.interner.definition_type(id);

    let mut string = String::new();
    string.push_str("    ");
    if definition_info.comptime {
        string.push_str("comptime ");
    }
    if expr_id.is_some() {
        string.push_str("let ");
    }
    if definition_info.mutable {
        string.push_str("mut ");
    }
    string.push_str(&definition_info.name);
    if !matches!(typ, Type::Error) {
        string.push_str(": ");
        string.push_str(&format!("{}", typ));
    }
    string
}

fn format_generics(generics: &Generics, string: &mut String) {
    if generics.is_empty() {
        return;
    }

    string.push('<');
    for (index, generic) in generics.iter().enumerate() {
        string.push_str(&generic.name);
        if index != generics.len() - 1 {
            string.push_str(", ");
        }
    }
    string.push('>');
}
fn format_pattern(pattern: &HirPattern, interner: &NodeInterner, string: &mut String) {
    match pattern {
        HirPattern::Identifier(ident) => {
            let definition = interner.definition(ident.id);
            string.push_str(&definition.name);
        }
        HirPattern::Mutable(pattern, _) => {
            string.push_str("mut ");
            format_pattern(pattern, interner, string);
        }
        HirPattern::Tuple(..) | HirPattern::Struct(..) => {
            string.push('_');
        }
    }
}

fn format_parent_module(
    referenced: ReferenceId,
    args: &ProcessRequestCallbackArgs,
    string: &mut String,
) -> bool {
    let Some(module) = args.interner.reference_module(referenced) else {
        return false;
    };

    return format_parent_module_from_module_id(&module, args, string);
}

fn format_parent_module_from_module_id(
    module: &ModuleId,
    args: &ProcessRequestCallbackArgs,
    string: &mut String,
) -> bool {
    let crate_id = module.krate;
    let crate_name = match crate_id {
        CrateId::Root(_) => Some(args.root_crate_name.clone()),
        CrateId::Crate(_) => args
            .root_crate_dependnencies
            .iter()
            .find(|dep| dep.crate_id == crate_id)
            .map(|dep| format!("{}", dep.name)),
        CrateId::Stdlib(_) => Some("std".to_string()),
        CrateId::Dummy => None,
    };

    let wrote_crate = if let Some(crate_name) = crate_name {
        string.push_str("    ");
        string.push_str(&crate_name);
        true
    } else {
        false
    };

    let Some(module_attributes) = args.interner.try_module_attributes(module) else {
        return wrote_crate;
    };

    if wrote_crate {
        string.push_str("::");
    }

    let mut segments = Vec::new();
    let mut current_attributes = module_attributes;
    while let Some(parent_attributes) = args.interner.try_module_attributes(&ModuleId {
        krate: module.krate,
        local_id: current_attributes.parent,
    }) {
        segments.push(&parent_attributes.name);
        current_attributes = parent_attributes;
    }

    for segment in segments.iter().rev() {
        string.push_str(segment);
        string.push_str("::");
    }

    string.push_str(&module_attributes.name);

    true
}
