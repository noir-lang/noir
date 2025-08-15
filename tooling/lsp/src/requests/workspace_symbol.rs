use std::{
    collections::{HashMap, HashSet},
    future::{self, Future},
    path::PathBuf,
};

use async_lsp::ResponseError;
use async_lsp::lsp_types;
use async_lsp::lsp_types::{
    SymbolKind, Url, WorkspaceSymbol, WorkspaceSymbolParams, WorkspaceSymbolResponse,
};
use fm::{FileManager, FileMap};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use nargo::{insert_all_files_under_path, parse_all};
use noirc_errors::{Location, Span};
use noirc_frontend::{
    ast::{
        Ident, LetStatement, NoirEnumeration, NoirFunction, NoirStruct, NoirTrait, NoirTraitImpl,
        Pattern, TraitImplItemKind, TraitItem, TypeAlias, TypeImpl, Visitor,
    },
    parser::ParsedSubModule,
};

use crate::{LspState, source_code_overrides};

use super::to_lsp_location;

pub(crate) fn on_workspace_symbol_request(
    state: &mut LspState,
    params: WorkspaceSymbolParams,
) -> impl Future<Output = Result<Option<WorkspaceSymbolResponse>, ResponseError>> + use<> {
    let Some(root_path) = state.root_path.clone() else {
        return future::ready(Ok(None));
    };

    let overrides = source_code_overrides(&state.input_files);

    // Prepare a FileManager for files we'll parse, to extract symbols from
    let mut file_manager = FileManager::new(root_path.as_path());

    let cache = &mut state.workspace_symbol_cache;

    // If the cache is not initialized yet, put all files in the workspace in the FileManager
    if !cache.initialized {
        insert_all_files_under_path(&mut file_manager, &root_path, Some(&overrides));
        cache.initialized = true;
    }

    // Then add files that we need to re-process
    for path in std::mem::take(&mut cache.paths_to_reprocess) {
        if !path.exists() {
            continue;
        }

        if let Some(source) = overrides.get(path.as_path()) {
            file_manager.add_file_with_source(path.as_path(), source.to_string());
        } else {
            let source = std::fs::read_to_string(path.as_path())
                .unwrap_or_else(|_| panic!("could not read file {path:?} into string"));
            file_manager.add_file_with_source(path.as_path(), source);
        }
    }

    // Note: what happens if a file is deleted? We don't get notifications when a file is deleted
    // so we might return symbols for a file that doesn't exist. However, VSCode seems to notice
    // the file doesn't exist and simply doesn't show the symbol. What if the file is re-created?
    // In that case we do get a notification so we'll reprocess that file.

    // Parse all files for which we don't know their symbols yet,
    // figure out the symbols and store them in the cache.
    let parsed_files = parse_all(&file_manager);
    for (file_id, (parsed_module, _)) in parsed_files {
        let path = file_manager.path(file_id).unwrap().to_path_buf();
        let mut gatherer = WorkspaceSymbolGatherer::new(file_manager.as_file_map());
        parsed_module.accept(&mut gatherer);
        cache.symbols_per_path.insert(path, gatherer.symbols);
    }

    // Finally, we filter the symbols based on the query
    // (Note: we could filter them as we gather them above, but doing this in one go is simpler)
    let matcher = SkimMatcherV2::default();
    let symbols = cache
        .symbols_per_path
        .values()
        .flat_map(|symbols| {
            symbols.iter().filter_map(|symbol| {
                if matcher.fuzzy_match(&symbol.name, &params.query).is_some() {
                    Some(symbol.clone())
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    future::ready(Ok(Some(WorkspaceSymbolResponse::Nested(symbols))))
}

#[derive(Default)]
pub(crate) struct WorkspaceSymbolCache {
    initialized: bool,
    symbols_per_path: HashMap<PathBuf, Vec<WorkspaceSymbol>>,
    /// Whenever a file changes we'll add it to this set. Then, when workspace symbols
    /// are requested we'll reprocess these files in search for symbols.
    paths_to_reprocess: HashSet<PathBuf>,
}

impl WorkspaceSymbolCache {
    pub(crate) fn reprocess_uri(&mut self, uri: &Url) {
        if !self.initialized {
            return;
        }

        if let Ok(path) = uri.to_file_path() {
            self.symbols_per_path.remove(&path);
            self.paths_to_reprocess.insert(path.clone());
        }
    }
}

struct WorkspaceSymbolGatherer<'files> {
    symbols: Vec<WorkspaceSymbol>,
    files: &'files FileMap,
}

impl<'files> WorkspaceSymbolGatherer<'files> {
    fn new(files: &'files FileMap) -> Self {
        Self { symbols: Vec::new(), files }
    }

    fn to_lsp_location(&self, location: Location) -> Option<lsp_types::Location> {
        to_lsp_location(self.files, location.file, location.span)
    }

    fn push_symbol(&mut self, name: &Ident, kind: SymbolKind) {
        self.push_symbol_impl(name, kind, None);
    }

    fn push_contained_symbol(&mut self, name: &Ident, kind: SymbolKind, container_name: String) {
        self.push_symbol_impl(name, kind, Some(container_name));
    }

    fn push_symbol_impl(&mut self, name: &Ident, kind: SymbolKind, container_name: Option<String>) {
        let Some(location) = self.to_lsp_location(name.location()) else {
            return;
        };

        let name = name.to_string();
        let location = lsp_types::OneOf::Left(location);
        let symbol =
            WorkspaceSymbol { name, kind, tags: None, container_name, location, data: None };
        self.symbols.push(symbol);
    }
}

impl Visitor for WorkspaceSymbolGatherer<'_> {
    fn visit_parsed_submodule(&mut self, submodule: &ParsedSubModule, _: Span) -> bool {
        self.push_symbol(&submodule.name, SymbolKind::MODULE);
        true
    }

    fn visit_noir_function(&mut self, noir_function: &NoirFunction, _span: Span) -> bool {
        self.push_symbol(noir_function.name_ident(), SymbolKind::FUNCTION);
        false
    }

    fn visit_noir_struct(&mut self, noir_struct: &NoirStruct, _span: Span) -> bool {
        self.push_symbol(&noir_struct.name, SymbolKind::STRUCT);
        false
    }

    fn visit_noir_enum(&mut self, noir_enum: &NoirEnumeration, _span: Span) -> bool {
        self.push_symbol(&noir_enum.name, SymbolKind::ENUM);
        false
    }

    fn visit_noir_type_alias(&mut self, alias: &TypeAlias, _span: Span) -> bool {
        self.push_symbol(&alias.name, SymbolKind::STRUCT);
        false
    }

    fn visit_noir_trait(&mut self, noir_trait: &NoirTrait, _: Span) -> bool {
        self.push_symbol(&noir_trait.name, SymbolKind::INTERFACE);

        let container_name = noir_trait.name.to_string();

        for item in &noir_trait.items {
            match &item.item {
                TraitItem::Function { name, .. } => {
                    self.push_contained_symbol(name, SymbolKind::FUNCTION, container_name.clone());
                }
                TraitItem::Constant { .. } | TraitItem::Type { .. } => (),
            }
        }

        false
    }

    fn visit_type_impl(&mut self, type_impl: &TypeImpl, _: Span) -> bool {
        let container_name = type_impl.object_type.to_string();

        for (method, _location) in &type_impl.methods {
            let method = &method.item;
            let kind = SymbolKind::FUNCTION;
            self.push_contained_symbol(method.name_ident(), kind, container_name.clone());
        }

        false
    }

    fn visit_noir_trait_impl(&mut self, trait_impl: &NoirTraitImpl, _: Span) -> bool {
        let container_name = trait_impl.object_type.to_string();

        for item in &trait_impl.items {
            match &item.item.kind {
                TraitImplItemKind::Function(noir_function) => {
                    let name = noir_function.name_ident();
                    let kind = SymbolKind::FUNCTION;
                    self.push_contained_symbol(name, kind, container_name.clone());
                }
                TraitImplItemKind::Constant(..) | TraitImplItemKind::Type { .. } => (),
            }
        }

        false
    }

    fn visit_global(&mut self, global: &LetStatement, _span: Span) -> bool {
        let Pattern::Identifier(name) = &global.pattern else {
            return false;
        };

        self.push_symbol(name, SymbolKind::CONSTANT);
        false
    }
}

#[cfg(test)]
mod tests {
    use async_lsp::lsp_types::{
        PartialResultParams, SymbolKind, WorkDoneProgressParams, WorkspaceSymbolParams,
        WorkspaceSymbolResponse,
    };
    use tokio::test;

    use crate::{on_workspace_symbol_request, test_utils};

    #[test]
    async fn test_workspace_symbol() {
        let (mut state, _) = test_utils::init_lsp_server("document_symbol").await;

        let response = on_workspace_symbol_request(
            &mut state,
            WorkspaceSymbolParams {
                query: String::new(),
                partial_result_params: PartialResultParams::default(),
                work_done_progress_params: WorkDoneProgressParams::default(),
            },
        )
        .await
        .expect("Could not execute on_document_symbol_request")
        .unwrap();

        let WorkspaceSymbolResponse::Nested(symbols) = response else {
            panic!("Expected Nested response, got {response:?}");
        };

        assert_eq!(symbols.len(), 8);

        assert_eq!(&symbols[0].name, "foo");
        assert_eq!(symbols[0].kind, SymbolKind::FUNCTION);
        assert!(symbols[0].container_name.is_none());

        assert_eq!(&symbols[1].name, "SomeStruct");
        assert_eq!(symbols[1].kind, SymbolKind::STRUCT);
        assert!(symbols[1].container_name.is_none());

        assert_eq!(&symbols[2].name, "new");
        assert_eq!(symbols[2].kind, SymbolKind::FUNCTION);
        assert_eq!(symbols[2].container_name.as_ref().unwrap(), "SomeStruct");

        assert_eq!(&symbols[3].name, "SomeTrait");
        assert_eq!(symbols[3].kind, SymbolKind::INTERFACE);
        assert!(symbols[3].container_name.is_none());

        assert_eq!(&symbols[4].name, "some_method");
        assert_eq!(symbols[4].kind, SymbolKind::FUNCTION);
        assert_eq!(symbols[4].container_name.as_ref().unwrap(), "SomeTrait");

        assert_eq!(&symbols[5].name, "some_method");
        assert_eq!(symbols[5].kind, SymbolKind::FUNCTION);
        assert_eq!(symbols[5].container_name.as_ref().unwrap(), "SomeStruct");

        assert_eq!(&symbols[6].name, "submodule");
        assert_eq!(symbols[6].kind, SymbolKind::MODULE);
        assert!(symbols[6].container_name.is_none());

        assert_eq!(&symbols[7].name, "SOME_GLOBAL");
        assert_eq!(symbols[7].kind, SymbolKind::CONSTANT);
        assert!(symbols[7].container_name.is_none());
    }
}
