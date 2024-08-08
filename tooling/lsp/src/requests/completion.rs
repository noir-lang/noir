use std::{
    collections::BTreeMap,
    future::{self, Future},
};

use async_lsp::ResponseError;
use fm::PathString;
use lsp_types::{CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse};
use noirc_errors::Span;
use noirc_frontend::{
    ast::{UseTree, UseTreeKind},
    graph::{CrateId, Dependency},
    hir::def_map::{CrateDefMap, ModuleId},
    parser::{Item, ItemKind},
    ParsedModule,
};

use crate::{utils, LspState};

use super::process_request;

pub(crate) fn on_completion_request(
    state: &mut LspState,
    params: CompletionParams,
) -> impl Future<Output = Result<Option<CompletionResponse>, ResponseError>> {
    noirc_frontend::log("Completion Request");

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

                let finder = NodeFinder::new(
                    byte_index,
                    byte,
                    args.root_crate_id,
                    args.def_maps,
                    args.root_crate_dependencies,
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
    krate: CrateId,
    current_module_id: ModuleId,
    def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
    dependencies: &'a Vec<Dependency>,
}

impl<'a> NodeFinder<'a> {
    fn new(
        byte_index: usize,
        byte: Option<u8>,
        krate: CrateId,
        def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
        dependencies: &'a Vec<Dependency>,
    ) -> Self {
        let def_map = &def_maps[&krate];
        let current_module_id = ModuleId { krate: krate, local_id: def_map.root() };
        Self { byte_index, byte, krate, current_module_id, def_maps, dependencies }
    }

    fn find(&self, parsed_module: &ParsedModule) -> Option<CompletionResponse> {
        for item in &parsed_module.items {
            if let Some(response) = self.find_in_item(item) {
                return Some(response);
            }
        }

        None
    }

    fn find_in_item(&self, item: &Item) -> Option<CompletionResponse> {
        if !self.includes_span(item.span) {
            return None;
        }

        match &item.kind {
            ItemKind::Import(use_tree) => {
                if let Some(completion) = self.find_in_use_tree(use_tree, item.span) {
                    return Some(completion);
                }
            }
            _ => (),
        }

        None
    }

    fn find_in_use_tree(&self, use_tree: &UseTree, span: Span) -> Option<CompletionResponse> {
        match &use_tree.kind {
            UseTreeKind::Path(ident, alias) => {
                if let Some(_alias) = alias {
                    // TODO: handle
                    None
                } else {
                    // If we are at the end of the use statement (likely the most common scenario)...
                    if self.byte_index == span.end() as usize {
                        if let Some(b':') = self.byte {
                            let mut segments: Vec<_> = use_tree
                                .prefix
                                .segments
                                .iter()
                                .map(|segment| &segment.ident)
                                .collect();
                            segments.push(ident);

                            // If we are after a colon, find inside the last segment
                            // TODO: handle
                            None
                        } else {
                            // Otherwise we must complete the last segment
                            if use_tree.prefix.segments.is_empty() {
                                // We are at the start of the use segment and completing the first segment,
                                // let's start here.
                                return self.complete_in_module(
                                    self.current_module_id,
                                    ident.to_string(),
                                    true,
                                );
                            }
                            None
                        }
                    } else {
                        // TODO: handle
                        None
                    }
                }
            }
            UseTreeKind::List(..) => {
                // TODO: handle
                None
            }
        }
    }

    fn complete_in_module(
        &self,
        module_id: ModuleId,
        prefix: String,
        first_segment: bool,
    ) -> Option<CompletionResponse> {
        let def_map = &self.def_maps[&module_id.krate];
        let module_data = def_map.modules().get(module_id.local_id.0)?;
        let mut completion_items = Vec::new();

        // Find in the target module
        for child in module_data.children.keys() {
            if name_matches(&child.0.contents, &prefix) {
                completion_items.push(module_completion_item(child.0.contents.clone()));
            }
        }

        // If this is the first segment, also find in all crates
        if first_segment {
            for dependency in self.dependencies {
                let dependency_name = dependency.as_name();
                if name_matches(&dependency_name, &prefix) {
                    completion_items.push(crate_completion_item(dependency_name));
                }
            }
        }

        Some(CompletionResponse::Array(completion_items))
    }

    fn includes_span(&self, span: Span) -> bool {
        span.start() as usize <= self.byte_index && self.byte_index <= span.end() as usize
    }
}

fn name_matches(name: &str, prefix: &str) -> bool {
    name.starts_with(prefix)
}

fn module_completion_item(name: String) -> CompletionItem {
    CompletionItem {
        label: name,
        label_details: None,
        kind: Some(CompletionItemKind::MODULE),
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

fn crate_completion_item(name: String) -> CompletionItem {
    module_completion_item(name)
}
