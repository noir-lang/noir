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

                let finder = NodeFinder::new(
                    byte_index,
                    byte,
                    args.root_crate_id,
                    args.def_maps,
                    args.root_crate_dependencies,
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
                self.find_in_use_tree_path(&use_tree.prefix, ident, alias, span)
            }
            UseTreeKind::List(..) => {
                // TODO: handle
                None
            }
        }
    }

    fn find_in_use_tree_path(
        &self,
        use_tree_prefix: &Path,
        ident: &Ident,
        alias: &Option<Ident>,
        span: Span,
    ) -> Option<CompletionResponse> {
        if let Some(_alias) = alias {
            // Won't handle completion if there's an alias (for now)
            return None;
        }

        if self.byte_index != span.end() as usize {
            // Won't handle autocomplete if we are not at the end of the use statement
            return None;
        }

        let mut segments: Vec<Ident> =
            use_tree_prefix.segments.iter().map(|segment| segment.ident.clone()).collect();

        if let Some(b':') = self.byte {
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
                self.complete_in_module(self.module_id, prefix, true)
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
        first_segment: bool,
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

fn module_completion_item(name: String) -> CompletionItem {
    simple_completion_item(name, CompletionItemKind::MODULE, None)
}

fn crate_completion_item(name: String) -> CompletionItem {
    simple_completion_item(name, CompletionItemKind::MODULE, None)
}

fn simple_completion_item(
    label: String,
    kind: CompletionItemKind,
    description: Option<String>,
) -> CompletionItem {
    CompletionItem {
        label,
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
