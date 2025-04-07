use std::{
    collections::HashMap,
    future::{self, Future},
    path::Path,
};

use async_lsp::ResponseError;
use convert_case::{Case, Casing};
use fm::{FILE_EXTENSION, FileManager, FileMap};
use lsp_types::{SymbolKind, WorkspaceSymbol, WorkspaceSymbolParams, WorkspaceSymbolResponse};
use nargo::parse_all;
use noirc_errors::{Location, Span};
use noirc_frontend::{
    ast::{
        Ident, LetStatement, NoirEnumeration, NoirFunction, NoirStruct, NoirTrait, NoirTraitImpl,
        NoirTypeAlias, Pattern, TraitImplItemKind, TraitItem, TypeImpl, Visitor,
    },
    parser::ParsedSubModule,
};
use walkdir::WalkDir;

use crate::{CachedWorkspaceSymbol, LspState, name_match::snake_name_matches};

use super::to_lsp_location;

pub(crate) fn on_workspace_symbol_request(
    state: &mut LspState,
    params: WorkspaceSymbolParams,
) -> impl Future<Output = Result<Option<WorkspaceSymbolResponse>, ResponseError>> + use<> {
    let Some(root_path) = state.root_path.clone() else {
        return future::ready(Ok(None));
    };

    let query = &params.query;
    let query_lowercase = query.to_lowercase();
    let query_snake_case = query.to_case(Case::Snake);

    // Source code for files we cached override those that are read from disk.
    let mut overrides: HashMap<&Path, &str> = HashMap::new();
    for (path, source) in &state.input_files {
        let path = path.strip_prefix("file://").unwrap();
        overrides.insert(Path::new(path), source);
    }

    let mut all_symbols = Vec::new();

    // Here we traverse all Noir files in the workspace, parsing those for which we don't know
    // their symbols. This isn't the best way to implement WorkspaceSymbol but it's actually pretty
    // fast, simple, and works very well. An alternative could be to process packages independently,
    // cache their symbols, and re-cache them or invalidate them on changes, but it's a much more complex
    // code that might yield marginal performance improvements.
    let mut file_manager = FileManager::new(root_path.as_path());
    for entry in WalkDir::new(root_path).sort_by_file_name() {
        let Ok(entry) = entry else {
            continue;
        };

        if !entry.file_type().is_file() {
            continue;
        }

        if entry.path().extension().is_none_or(|ext| ext != FILE_EXTENSION) {
            continue;
        };

        let path = entry.into_path();
        if let Some(symbols) = state.workspace_symbol_cache.get(&path) {
            all_symbols.extend(symbols.clone());
        } else {
            let source = if let Some(src) = overrides.get(path.as_path()) {
                src.to_string()
            } else {
                std::fs::read_to_string(path.as_path())
                    .unwrap_or_else(|_| panic!("could not read file {:?} into string", path))
            };

            file_manager.add_file_with_source(path.as_path(), source);
        }
    }

    let parsed_files = parse_all(&file_manager);

    for (file_id, (parsed_module, _)) in parsed_files {
        let path = file_manager.path(file_id).unwrap().to_path_buf();
        let mut symbols = Vec::new();
        let mut gatherer = WorkspaceSymboGatherer::new(&mut symbols, file_manager.as_file_map());
        parsed_module.accept(&mut gatherer);
        state.workspace_symbol_cache.insert(path, gatherer.symbols.clone());
        all_symbols.extend(std::mem::take(gatherer.symbols));
    }

    let symbols = all_symbols
        .into_iter()
        .filter_map(|symbol| {
            let name = &symbol.symbol.name;
            let name_lowercase = name.to_lowercase();
            (name_lowercase.contains(&query_lowercase)
                || snake_name_matches(&symbol.name_in_snake_case, &query_snake_case))
            .then_some(symbol.symbol)
        })
        .collect::<Vec<_>>();

    future::ready(Ok(Some(WorkspaceSymbolResponse::Nested(symbols))))
}

struct WorkspaceSymboGatherer<'symbols, 'files> {
    symbols: &'symbols mut Vec<CachedWorkspaceSymbol>,
    files: &'files FileMap,
}

impl<'symbols, 'files> WorkspaceSymboGatherer<'symbols, 'files> {
    fn new(symbols: &'symbols mut Vec<CachedWorkspaceSymbol>, files: &'files FileMap) -> Self {
        Self { symbols, files }
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
        let symbol = WorkspaceSymbol {
            name: name.clone(),
            kind,
            tags: None,
            container_name,
            location: lsp_types::OneOf::Left(location),
            data: None,
        };
        let cached_symbol =
            CachedWorkspaceSymbol { symbol, name_in_snake_case: name.to_case(Case::Snake) };
        self.symbols.push(cached_symbol);
    }
}

impl Visitor for WorkspaceSymboGatherer<'_, '_> {
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

    fn visit_noir_type_alias(&mut self, alias: &NoirTypeAlias, _span: Span) -> bool {
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
    use lsp_types::{
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
            panic!("Expected Nested response, got {:?}", response);
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
