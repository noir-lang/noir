use std::future::{self, Future};

use async_lsp::ResponseError;
use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind};
use noirc_frontend::{
    ast::Visibility,
    hir::def_map::ModuleId,
    hir_def::stmt::HirPattern,
    macros_api::{NodeInterner, StructId},
    node_interner::{
        DefinitionId, DefinitionKind, FuncId, GlobalId, ReferenceId, TraitId, TypeAliasId,
    },
    Generics, Type,
};

use crate::LspState;

use super::{process_request, to_lsp_location};

pub(crate) fn on_hover_request(
    state: &mut LspState,
    params: HoverParams,
) -> impl Future<Output = Result<Option<Hover>, ResponseError>> {
    let result = process_request(
        state,
        params.text_document_position_params,
        |location, interner, files, _| {
            interner.reference_at_location(location).map(|reference| {
                let location = interner.reference_location(reference);
                let lsp_location = to_lsp_location(files, location.file, location.span);
                Hover {
                    range: lsp_location.map(|location| location.range),
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format_reference(reference, interner),
                    }),
                }
            })
        },
    );

    future::ready(result)
}

fn format_reference(reference: ReferenceId, interner: &NodeInterner) -> String {
    match reference {
        ReferenceId::Module(id) => format_module(id, interner),
        ReferenceId::Struct(id) => format_struct(id, interner),
        ReferenceId::StructMember(id, field_index) => {
            format_struct_member(id, field_index, interner)
        }
        ReferenceId::Trait(id) => format_trait(id, interner),
        ReferenceId::Global(id) => format_global(id, interner),
        ReferenceId::Function(id) => format_function(id, interner),
        ReferenceId::Alias(id) => format_alias(id, interner),
        ReferenceId::Local(id) => format_local(id, &interner),
        ReferenceId::Reference(location, _) => {
            format_reference(interner.find_referenced(location).unwrap(), interner)
        }
    }
}
fn format_module(id: ModuleId, interner: &NodeInterner) -> String {
    let name = &interner.module_attributes(&id).name;

    let mut string = String::new();
    // TODO: append the module path
    string.push_str("    ");
    string.push_str("mod ");
    string.push_str(name);
    string
}

fn format_struct(id: StructId, interner: &NodeInterner) -> String {
    let struct_type = interner.get_struct(id);
    let struct_type = struct_type.borrow();

    let mut string = String::new();
    // TODO: append the module path
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

fn format_struct_member(id: StructId, field_index: usize, interner: &NodeInterner) -> String {
    let struct_type = interner.get_struct(id);
    let struct_type = struct_type.borrow();
    let (field_name, field_type) = struct_type.field_at(field_index);

    let mut string = String::new();
    // TODO: append the module path
    string.push_str("    ");
    string.push_str(&field_name.0.contents);
    string.push_str(": ");
    string.push_str(&format!("{}", field_type));
    string
}

fn format_trait(id: TraitId, interner: &NodeInterner) -> String {
    let a_trait = interner.get_trait(id);

    let mut string = String::new();
    // TODO: append the module path
    string.push_str("    ");
    string.push_str("trait ");
    string.push_str(&a_trait.name.0.contents);
    format_generics(&a_trait.generics, &mut string);
    string
}

fn format_global(id: GlobalId, interner: &NodeInterner) -> String {
    let global_info = interner.get_global(id);
    let definition_id = global_info.definition_id;
    let typ = interner.definition_type(definition_id);

    let mut string = String::new();
    // TODO: append the module path
    string.push_str("    ");
    string.push_str("global ");
    string.push_str(&global_info.ident.0.contents);
    string.push_str(": ");
    string.push_str(&format!("{}", typ));
    string
}

fn format_function(id: FuncId, interner: &NodeInterner) -> String {
    let func_meta = interner.function_meta(&id);
    let func_name_definition_id = interner.definition(func_meta.name.id);

    let mut string = String::new();
    // TODO: append the module path
    string.push_str("    ");
    string.push_str("fn ");
    format_generics(&func_meta.direct_generics, &mut string);
    string.push_str(&func_name_definition_id.name);
    string.push('(');
    let parameters = &func_meta.parameters;
    for (index, (pattern, typ, visibility)) in parameters.iter().enumerate() {
        format_pattern(pattern, interner, &mut string);
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

fn format_alias(id: TypeAliasId, interner: &NodeInterner) -> String {
    let type_alias = interner.get_type_alias(id);
    let type_alias = type_alias.borrow();

    let mut string = String::new();
    // TODO: append the module path
    string.push_str("    ");
    string.push_str("type ");
    string.push_str(&type_alias.name.0.contents);
    string.push_str(" = ");
    string.push_str(&format!("{}", &type_alias.typ));
    string
}

fn format_local(id: DefinitionId, interner: &NodeInterner) -> String {
    let definition_info = interner.definition(id);
    let DefinitionKind::Local(expr_id) = definition_info.kind else {
        panic!("Expected a local reference to reference a local definition")
    };
    let typ = interner.definition_type(id);

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
