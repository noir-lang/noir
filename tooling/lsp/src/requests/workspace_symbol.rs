use std::{
    collections::HashMap,
    future::{self, Future},
    path::{Path, PathBuf},
};

use async_lsp::ResponseError;
use convert_case::{Case, Casing};
use fm::{FileManager, FileMap};
use lsp_types::{SymbolKind, WorkspaceSymbol, WorkspaceSymbolParams, WorkspaceSymbolResponse};
use nargo::{insert_all_files_under_path_into_file_manager, parse_all};
use noirc_errors::{Location, Span};
use noirc_frontend::ast::{
    Ident, LetStatement, ModuleDeclaration, NoirEnumeration, NoirFunction, NoirStruct, NoirTrait,
    NoirTraitImpl, NoirTypeAlias, Pattern, TraitImplItemKind, TraitItem, TypeImpl, Visitor,
};

use crate::{LspState, name_match::name_matches};

use super::to_lsp_location;

pub(crate) fn on_workspace_symbol_request(
    state: &mut LspState,
    params: WorkspaceSymbolParams,
) -> impl Future<Output = Result<Option<WorkspaceSymbolResponse>, ResponseError>> + use<> {
    let Some(root_path) = state.root_path.clone() else {
        return future::ready(Ok(None));
    };

    let now = std::time::Instant::now();

    let query = &params.query;
    let query_lowercase = query.to_lowercase();
    let query_snake_case = query.to_case(Case::Snake);

    let file_manager = setup_file_manager(state, root_path);
    let parsed_files = parse_all(&file_manager);

    let mut symbols = Vec::new();

    for (_, (parsed_module, _)) in parsed_files {
        let mut gatherer = WorkspaceSymboGatherer::new(&mut symbols, file_manager.as_file_map());
        parsed_module.accept(&mut gatherer);
    }

    let symbols = symbols
        .into_iter()
        .filter(|symbol| {
            let name = &symbol.name;
            let name_lowercase = name.to_lowercase();
            name_lowercase.contains(&query_lowercase) || name_matches(name, &query_snake_case)
        })
        .collect::<Vec<_>>();

    let elapsed = now.elapsed();
    eprintln!("Elapsed: {:.2?}", elapsed);

    future::ready(Ok(Some(WorkspaceSymbolResponse::Nested(symbols))))
}

fn setup_file_manager(state: &mut LspState, root_path: PathBuf) -> FileManager {
    let mut file_manager = FileManager::new(root_path.as_path());

    // Source code for files we cached override those that are read from disk.
    let mut overrides: HashMap<&Path, &str> = HashMap::new();
    for (path, source) in &state.input_files {
        let path = path.strip_prefix("file://").unwrap();
        overrides.insert(Path::new(path), source);
    }

    insert_all_files_under_path_into_file_manager(&mut file_manager, &root_path, &overrides);

    file_manager
}

struct WorkspaceSymboGatherer<'symbols, 'files> {
    symbols: &'symbols mut Vec<WorkspaceSymbol>,
    files: &'files FileMap,
}

impl<'symbols, 'files> WorkspaceSymboGatherer<'symbols, 'files> {
    fn new(symbols: &'symbols mut Vec<WorkspaceSymbol>, files: &'files FileMap) -> Self {
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

        self.symbols.push(WorkspaceSymbol {
            name: name.to_string(),
            kind,
            tags: None,
            container_name,
            location: lsp_types::OneOf::Left(location),
            data: None,
        });
    }
}

impl Visitor for WorkspaceSymboGatherer<'_, '_> {
    fn visit_module_declaration(&mut self, module: &ModuleDeclaration, _span: Span) {
        self.push_symbol(&module.ident, SymbolKind::MODULE);
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
