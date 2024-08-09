use std::{
    collections::BTreeMap,
    future::{self, Future},
};

use async_lsp::ResponseError;
use fm::PathString;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionParams,
    CompletionResponse,
};
use noirc_errors::Span;
use noirc_frontend::{
    ast::{Ident, Path, PathKind, PathSegment, UseTree, UseTreeKind},
    graph::{CrateId, Dependency},
    hir::{
        def_map::{CrateDefMap, ModuleId},
        resolution::path_resolver::{PathResolver, StandardPathResolver},
    },
    macros_api::{ModuleDefId, NodeInterner, StructId},
    node_interner::{FuncId, GlobalId, TraitId, TypeAliasId},
    parser::{Item, ItemKind},
    ParsedModule, Type,
};

use crate::{utils, LspState};

use super::process_request;

pub(crate) fn on_completion_request(
    state: &mut LspState,
    params: CompletionParams,
) -> impl Future<Output = Result<Option<CompletionResponse>, ResponseError>> {
    let uri = params.text_document_position.clone().text_document.uri;

    let result = process_request(state, params.text_document_position.clone(), |args| {
        let path = PathString::from_path(uri.to_file_path().unwrap());
        args.files.get_file_id(&path).and_then(|file_id| {
            utils::position_to_byte_index(
                args.files,
                file_id,
                &params.text_document_position.position,
            )
            .and_then(|byte_index| {
                let file = args.files.get_file(file_id).unwrap();
                let source = file.source();
                let byte = source.bytes().nth(byte_index - 1);
                let (parsed_module, _errors) = noirc_frontend::parse_program(source);

                let mut finder = NodeFinder::new(
                    byte_index,
                    byte,
                    args.crate_id,
                    args.def_maps,
                    args.dependencies,
                    args.interner,
                );
                finder.find(&parsed_module)
            })
        })
    });
    future::ready(result)
}

struct NodeFinder<'a> {
    byte_index: usize,
    byte: Option<u8>,
    module_id: ModuleId,
    def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
    dependencies: &'a Vec<Dependency>,
    interner: &'a NodeInterner,
}

impl<'a> NodeFinder<'a> {
    fn new(
        byte_index: usize,
        byte: Option<u8>,
        krate: CrateId,
        def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
        dependencies: &'a Vec<Dependency>,
        interner: &'a NodeInterner,
    ) -> Self {
        let def_map = &def_maps[&krate];
        let current_module_id = ModuleId { krate: krate, local_id: def_map.root() };
        Self { byte_index, byte, module_id: current_module_id, def_maps, dependencies, interner }
    }

    fn find(&mut self, parsed_module: &ParsedModule) -> Option<CompletionResponse> {
        self.find_in_parsed_module(parsed_module)
    }

    fn find_in_parsed_module(
        &mut self,
        parsed_module: &ParsedModule,
    ) -> Option<CompletionResponse> {
        for item in &parsed_module.items {
            if let Some(response) = self.find_in_item(item) {
                return Some(response);
            }
        }

        None
    }

    fn find_in_item(&mut self, item: &Item) -> Option<CompletionResponse> {
        if !self.includes_span(item.span) {
            return None;
        }

        match &item.kind {
            ItemKind::Import(use_tree) => {
                let mut prefixes = Vec::new();
                if let Some(completion) = self.find_in_use_tree(use_tree, &mut prefixes) {
                    return Some(completion);
                }
            }
            ItemKind::Submodules(parsed_sub_module) => {
                // Switch `self.module_id` to the submodule
                let previous_module_id = self.module_id;

                let def_map = &self.def_maps[&self.module_id.krate];
                let module_data = def_map.modules().get(self.module_id.local_id.0)?;
                if let Some(child_module) = module_data.children.get(&parsed_sub_module.name) {
                    self.module_id =
                        ModuleId { krate: self.module_id.krate, local_id: *child_module };
                }

                let completion = self.find_in_parsed_module(&parsed_sub_module.contents);

                // Restore the old module before continuing
                self.module_id = previous_module_id;

                if let Some(completion) = completion {
                    return Some(completion);
                }
            }
            _ => (),
        }

        None
    }

    fn find_in_use_tree(
        &self,
        use_tree: &UseTree,
        prefixes: &mut Vec<Path>,
    ) -> Option<CompletionResponse> {
        match &use_tree.kind {
            UseTreeKind::Path(ident, alias) => {
                prefixes.push(use_tree.prefix.clone());
                let response = self.find_in_use_tree_path(&prefixes, ident, alias);
                prefixes.pop();
                response
            }
            UseTreeKind::List(use_trees) => {
                prefixes.push(use_tree.prefix.clone());
                for use_tree in use_trees {
                    if let Some(completion) = self.find_in_use_tree(use_tree, prefixes) {
                        return Some(completion);
                    }
                }
                prefixes.pop();
                None
            }
        }
    }

    fn find_in_use_tree_path(
        &self,
        prefixes: &Vec<Path>,
        ident: &Ident,
        alias: &Option<Ident>,
    ) -> Option<CompletionResponse> {
        if let Some(_alias) = alias {
            // Won't handle completion if there's an alias (for now)
            return None;
        }

        let after_colons = self.byte == Some(b':');
        let at_ident_end = self.byte_index == ident.span().end() as usize;
        let at_ident_colons_end =
            after_colons && self.byte_index - 2 == ident.span().end() as usize;

        if !(at_ident_end || at_ident_colons_end) {
            return None;
        }

        let mut segments: Vec<Ident> = Vec::new();
        for prefix in prefixes {
            for segment in &prefix.segments {
                segments.push(segment.ident.clone());
            }
        }

        if after_colons {
            // We are after the colon
            segments.push(ident.clone());

            self.resolve_module(segments).and_then(|module_id| {
                let prefix = String::new();
                self.complete_in_module(module_id, prefix, false)
            })
        } else {
            let prefix = ident.to_string();

            // Otherwise we must complete the last segment
            if segments.is_empty() {
                // We are at the start of the use segment and completing the first segment
                self.complete_in_module(
                    self.module_id,
                    prefix,
                    prefixes.first().unwrap().kind != PathKind::Crate,
                )
            } else {
                self.resolve_module(segments)
                    .and_then(|module_id| self.complete_in_module(module_id, prefix, false))
            }
        }
    }

    fn complete_in_module(
        &self,
        module_id: ModuleId,
        prefix: String,
        suggest_crates: bool,
    ) -> Option<CompletionResponse> {
        let def_map = &self.def_maps[&module_id.krate];
        let module_data = def_map.modules().get(module_id.local_id.0)?;
        let mut completion_items = Vec::new();

        for ident in module_data.definitions().names() {
            let name = &ident.0.contents;

            if name_matches(name, &prefix) {
                let per_ns = module_data.find_name(ident);
                if let Some((module_def_id, _, _)) = per_ns.types {
                    completion_items
                        .push(self.module_def_id_completion_item(module_def_id, name.clone()));
                }

                if let Some((module_def_id, _, _)) = per_ns.values {
                    completion_items
                        .push(self.module_def_id_completion_item(module_def_id, name.clone()));
                }
            }
        }

        // If this is the first segment, also find in all crates
        if suggest_crates {
            for dependency in self.dependencies {
                let dependency_name = dependency.as_name();
                if name_matches(&dependency_name, &prefix) {
                    completion_items.push(crate_completion_item(dependency_name));
                }
            }

            if name_matches("crate::", &prefix) {
                completion_items.push(simple_completion_item(
                    "crate::",
                    CompletionItemKind::KEYWORD,
                    None,
                ));
            }

            if module_data.parent.is_some() && name_matches("super::", &prefix) {
                completion_items.push(simple_completion_item(
                    "super::",
                    CompletionItemKind::KEYWORD,
                    None,
                ));
            }
        }

        Some(CompletionResponse::Array(completion_items))
    }

    fn module_def_id_completion_item(
        &self,
        module_def_id: ModuleDefId,
        name: String,
    ) -> CompletionItem {
        match module_def_id {
            ModuleDefId::ModuleId(_) => module_completion_item(name),
            ModuleDefId::FunctionId(func_id) => self.function_completion_item(func_id),
            ModuleDefId::TypeId(struct_id) => self.struct_completion_item(struct_id),
            ModuleDefId::TypeAliasId(type_alias_id) => {
                self.type_alias_completion_item(type_alias_id)
            }
            ModuleDefId::TraitId(trait_id) => self.trait_completion_item(trait_id),
            ModuleDefId::GlobalId(global_id) => self.global_completion_item(global_id),
        }
    }

    fn function_completion_item(&self, func_id: FuncId) -> CompletionItem {
        let name = self.interner.function_name(&func_id).to_string();
        let mut typ = &self.interner.function_meta(&func_id).typ;
        if let Type::Forall(_, typ_) = typ {
            typ = typ_;
        }
        let description = typ.to_string();

        simple_completion_item(name, CompletionItemKind::FUNCTION, Some(description))
    }

    fn struct_completion_item(&self, struct_id: StructId) -> CompletionItem {
        let struct_type = self.interner.get_struct(struct_id);
        let struct_type = struct_type.borrow();
        let name = struct_type.name.to_string();

        simple_completion_item(name.clone(), CompletionItemKind::STRUCT, Some(name))
    }

    fn type_alias_completion_item(&self, type_alias_id: TypeAliasId) -> CompletionItem {
        let type_alias = self.interner.get_type_alias(type_alias_id);
        let type_alias = type_alias.borrow();
        let name = type_alias.name.to_string();

        simple_completion_item(name.clone(), CompletionItemKind::STRUCT, Some(name))
    }

    fn trait_completion_item(&self, trait_id: TraitId) -> CompletionItem {
        let trait_ = self.interner.get_trait(trait_id);
        let name = trait_.name.to_string();

        simple_completion_item(name.clone(), CompletionItemKind::INTERFACE, Some(name))
    }

    fn global_completion_item(&self, global_id: GlobalId) -> CompletionItem {
        let global_definition = self.interner.get_global_definition(global_id);
        let name = global_definition.name.clone();

        let global = self.interner.get_global(global_id);
        let typ = self.interner.definition_type(global.definition_id);
        let description = typ.to_string();

        simple_completion_item(name, CompletionItemKind::CONSTANT, Some(description))
    }

    fn resolve_module(&self, segments: Vec<Ident>) -> Option<ModuleId> {
        if let Some(ModuleDefId::ModuleId(module_id)) = self.resolve_path(segments) {
            Some(module_id)
        } else {
            None
        }
    }

    fn resolve_path(&self, segments: Vec<Ident>) -> Option<ModuleDefId> {
        let path_segments = segments.into_iter().map(PathSegment::from).collect();
        let path = Path { segments: path_segments, kind: PathKind::Plain, span: Span::default() };

        let path_resolver = StandardPathResolver::new(self.module_id);
        match path_resolver.resolve(&self.def_maps, path, &mut None) {
            Ok(path_resolution) => Some(path_resolution.module_def_id),
            Err(_) => None,
        }
    }

    fn includes_span(&self, span: Span) -> bool {
        span.start() as usize <= self.byte_index && self.byte_index <= span.end() as usize
    }
}

fn name_matches(name: &str, prefix: &str) -> bool {
    name.starts_with(prefix)
}

fn module_completion_item(name: impl Into<String>) -> CompletionItem {
    simple_completion_item(name.into(), CompletionItemKind::MODULE, None)
}

fn crate_completion_item(name: String) -> CompletionItem {
    simple_completion_item(name, CompletionItemKind::MODULE, None)
}

fn simple_completion_item(
    label: impl Into<String>,
    kind: CompletionItemKind,
    description: Option<String>,
) -> CompletionItem {
    CompletionItem {
        label: label.into(),
        label_details: Some(CompletionItemLabelDetails { detail: None, description: description }),
        kind: Some(kind),
        detail: None,
        documentation: None,
        deprecated: None,
        preselect: None,
        sort_text: None,
        filter_text: None,
        insert_text: None,
        insert_text_format: None,
        insert_text_mode: None,
        text_edit: None,
        additional_text_edits: None,
        command: None,
        commit_characters: None,
        data: None,
        tags: None,
    }
}

#[cfg(test)]
mod completion_tests {
    use crate::{notifications::on_did_open_text_document, test_utils};

    use super::*;
    use lsp_types::{
        DidOpenTextDocumentParams, PartialResultParams, Position, TextDocumentIdentifier,
        TextDocumentItem, TextDocumentPositionParams, WorkDoneProgressParams,
    };
    use tokio::test;

    async fn assert_completion(src: &str, expected: Vec<CompletionItem>) {
        let (mut state, noir_text_document) = test_utils::init_lsp_server("document_symbol").await;

        let (line, column) = src
            .lines()
            .enumerate()
            .filter_map(|(line_index, line)| {
                line.find(">|<").map(|char_index| (line_index, char_index))
            })
            .nth(0)
            .expect("Expected to find one >|< in the source code");

        let src = src.replace(">|<", "");

        on_did_open_text_document(
            &mut state,
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: noir_text_document.clone(),
                    language_id: "noir".to_string(),
                    version: 0,
                    text: src.to_string(),
                },
            },
        );

        // Get inlay hints. These should now be relative to the changed text,
        // not the saved file's text.
        let response = on_completion_request(
            &mut state,
            CompletionParams {
                text_document_position: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: noir_text_document },
                    position: Position { line: line as u32, character: column as u32 },
                },
                work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
                partial_result_params: PartialResultParams { partial_result_token: None },
                context: None,
            },
        )
        .await
        .expect("Could not execute on_completion_request")
        .unwrap();

        let CompletionResponse::Array(items) = response else {
            panic!("Expected response to be CompletionResponse::Array");
        };

        let mut items = items.clone();
        items.sort_by_key(|item| item.label.clone());

        let mut expected = expected.clone();
        expected.sort_by_key(|item| item.label.clone());

        if items != expected {
            println!(
                "Items: {:?}",
                items.iter().map(|item| item.label.clone()).collect::<Vec<_>>()
            );
            println!(
                "Expected: {:?}",
                expected.iter().map(|item| item.label.clone()).collect::<Vec<_>>()
            );
        }

        assert_eq!(items, expected);
    }

    #[test]
    async fn test_use_first_segment() {
        let src = r#"
            mod foo {}
            mod foobar {}
            use f>|<
        "#;

        assert_completion(
            src,
            vec![module_completion_item("foo"), module_completion_item("foobar")],
        )
        .await;
    }

    #[test]
    async fn test_use_second_segment() {
        let src = r#"
            mod foo {
                mod bar {}
                mod baz {}
            }
            use foo::>|<
        "#;

        assert_completion(src, vec![module_completion_item("bar"), module_completion_item("baz")])
            .await;
    }

    #[test]
    async fn test_use_second_segment_after_typing() {
        let src = r#"
            mod foo {
                mod bar {}
                mod brave {}
            }
            use foo::ba>|<
        "#;

        assert_completion(src, vec![module_completion_item("bar")]).await;
    }

    #[test]
    async fn test_use_struct() {
        let src = r#"
            mod foo {
                struct Foo {}
            }
            use foo::>|<
        "#;

        assert_completion(
            src,
            vec![simple_completion_item(
                "Foo",
                CompletionItemKind::STRUCT,
                Some("Foo".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_use_function() {
        let src = r#"
            mod foo {
                fn bar(x: i32) -> u64 { 0 }
            }
            use foo::>|<
        "#;

        assert_completion(
            src,
            vec![simple_completion_item(
                "bar",
                CompletionItemKind::FUNCTION,
                Some("fn(i32) -> u64".to_string()),
            )],
        )
        .await;
    }

    #[test]
    async fn test_use_after_crate_and_letter() {
        let src = r#"
            mod foo {}
            use crate::f>|<
        "#;

        assert_completion(src, vec![module_completion_item("foo")]).await;
    }

    #[test]
    async fn test_use_suggests_hardcoded_crate() {
        let src = r#"
            use c>|<
        "#;

        assert_completion(
            src,
            vec![simple_completion_item("crate::", CompletionItemKind::KEYWORD, None)],
        )
        .await;
    }

    #[test]
    async fn test_use_in_tree_after_letter() {
        let src = r#"
            mod foo {
                mod bar {}
            }
            use foo::{b>|<}
        "#;

        assert_completion(src, vec![module_completion_item("bar")]).await;
    }

    #[test]
    async fn test_use_in_tree_after_colons() {
        let src = r#"
            mod foo {
                mod bar {
                    mod baz {}
                }
            }
            use foo::{bar::>|<}
        "#;

        assert_completion(src, vec![module_completion_item("baz")]).await;
    }

    #[test]
    async fn test_use_in_tree_after_colons_after_another_segment() {
        let src = r#"
            mod foo {
                mod bar {}
                mod qux {}
            }
            use foo::{bar, q>|<}
        "#;

        assert_completion(src, vec![module_completion_item("qux")]).await;
    }

    #[test]
    async fn test_use_in_nested_module() {
        let src = r#"
            mod foo {
                mod something {}

                use s>|<
            }
        "#;

        assert_completion(
            src,
            vec![
                module_completion_item("something"),
                module_completion_item("std"),
                simple_completion_item("super::", CompletionItemKind::KEYWORD, None),
            ],
        )
        .await;
    }
}
