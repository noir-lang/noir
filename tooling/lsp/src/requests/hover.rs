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
        ReferenceId::Local(id) => format_local(id, args),
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
        string.push('\n');
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
        string.push('\n');
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
    string.push('\n');
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
        string.push('\n');
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
        string.push('\n');
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
        string.push('\n');
    }
    string.push_str("    ");
    string.push_str("fn ");
    string.push_str(&func_name_definition_id.name);
    format_generics(&func_meta.direct_generics, &mut string);
    string.push('(');
    let parameters = &func_meta.parameters;
    for (index, (pattern, typ, visibility)) in parameters.iter().enumerate() {
        format_pattern(pattern, args.interner, &mut string);
        if !pattern_is_self(pattern, args.interner) {
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

    string
}

fn format_alias(id: TypeAliasId, args: &ProcessRequestCallbackArgs) -> String {
    let type_alias = args.interner.get_type_alias(id);
    let type_alias = type_alias.borrow();

    let mut string = String::new();
    format_parent_module(ReferenceId::Alias(id), args, &mut string);
    string.push('\n');
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
        if expr_id.is_none() {
            string.push_str("let ");
        }
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

fn format_parent_module(
    referenced: ReferenceId,
    args: &ProcessRequestCallbackArgs,
    string: &mut String,
) -> bool {
    let Some(module) = args.interner.reference_module(referenced) else {
        return false;
    };

    format_parent_module_from_module_id(module, args, string)
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
            .root_crate_dependencies
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
    } else {
        string.push_str("    ");
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

#[cfg(test)]
mod hover_tests {
    use crate::test_utils;

    use super::*;
    use lsp_types::{
        Position, TextDocumentIdentifier, TextDocumentPositionParams, Url, WorkDoneProgressParams,
    };
    use tokio::test;

    async fn assert_hover(directory: &str, file: &str, position: Position, expected_text: &str) {
        let (mut state, noir_text_document) = test_utils::init_lsp_server(directory).await;

        // noir_text_document is always `src/main.nr` in the workspace directory, so let's go to the workspace dir
        let noir_text_document = noir_text_document.to_file_path().unwrap();
        let workspace_dir = noir_text_document.parent().unwrap().parent().unwrap();

        let file_uri = Url::from_file_path(workspace_dir.join(file)).unwrap();

        let hover = on_hover_request(
            &mut state,
            HoverParams {
                text_document_position_params: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: file_uri },
                    position,
                },
                work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
            },
        )
        .await
        .expect("Could not execute hover")
        .unwrap();

        let HoverContents::Markup(markup) = hover.contents else {
            panic!("Expected hover contents to be Markup");
        };

        assert_eq!(markup.value, expected_text);
    }

    #[test]
    async fn hover_on_module() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 6, character: 9 },
            r#"    one
    mod subone"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_struct() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 9, character: 20 },
            r#"    one::subone
    struct SubOneStruct {
        some_field: i32,
        some_other_field: Field,
    }"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_generic_struct() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 46, character: 17 },
            r#"    one::subone
    struct GenericStruct<A, B> {
    }"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_struct_member() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 9, character: 35 },
            r#"    one::subone::SubOneStruct
    some_field: i32"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_trait() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 12, character: 17 },
            r#"    one::subone
    trait SomeTrait"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_global() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 15, character: 25 },
            r#"    one::subone
    global some_global: Field"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_function() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 3, character: 4 },
            r#"    one
    fn function_one<A, B>()"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_local_function() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 2, character: 7 },
            r#"    two
    fn function_two()"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_struct_method() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 20, character: 6 },
            r#"    one::subone::SubOneStruct
    fn foo(self, x: i32, y: i32) -> Field"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_local_var() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 25, character: 12 },
            "    let regular_var: Field",
        )
        .await;
    }

    #[test]
    async fn hover_on_local_mut_var() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 27, character: 4 },
            "    let mut mutable_var: Field",
        )
        .await;
    }

    #[test]
    async fn hover_on_parameter() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 31, character: 12 },
            "    some_param: i32",
        )
        .await;
    }

    #[test]
    async fn hover_on_alias() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 34, character: 17 },
            r#"    one::subone
    type SomeAlias = i32"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_trait_on_call() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 39, character: 17 },
            r#"    std::default
    trait Default"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_std_module_in_use() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 36, character: 9 },
            r#"    std
    mod default"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_crate_module_in_call() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 15, character: 17 },
            r#"    one
    mod subone"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_module_without_crate_or_std_prefix() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 43, character: 4 },
            r#"    two
    mod other"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_module_with_crate_prefix() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 44, character: 11 },
            r#"    two
    mod other"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_module_on_struct_constructor() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 19, character: 12 },
            r#"    one
    mod subone"#,
        )
        .await;
    }

    #[test]
    async fn hover_on_type_inside_generic_arguments() {
        assert_hover(
            "workspace",
            "two/src/lib.nr",
            Position { line: 51, character: 30 },
            r#"    one::subone
    struct SubOneStruct {
        some_field: i32,
        some_other_field: Field,
    }"#,
        )
        .await;
    }
}
