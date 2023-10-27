use std::future::{self, Future};

use async_lsp::{ErrorCode, ResponseError};
use lsp_types::{DocumentSymbol, SymbolKind};
use noirc_frontend::{
    parse_program, parser::ItemKind, NoirFunction, Pattern, TraitImplItem, TraitItem,
};

use crate::{
    types::{DocumentSymbolParams, DocumentSymbolResponse},
    LspState,
};

pub(crate) fn on_document_symbols(
    state: &mut LspState,
    params: DocumentSymbolParams,
) -> impl Future<Output = Result<Option<DocumentSymbolResponse>, ResponseError>> {
    future::ready(on_document_symbols_inner(state, params))
}

fn on_document_symbols_inner(
    state: &mut LspState,
    params: DocumentSymbolParams,
) -> Result<Option<DocumentSymbolResponse>, ResponseError> {
    let text = if state.open_files.contains_key(&params.text_document.uri) {
        state.open_files.get(&params.text_document.uri).unwrap().clone()
    } else {
        std::fs::read_to_string(params.text_document.uri.path()).map_err(|err| {
            ResponseError::new(ErrorCode::REQUEST_FAILED, format!("Could not read file: {err}"))
        })?
    };

    let (parsed_module, _) = parse_program(text.as_str());
    let symbols: Vec<DocumentSymbol> =
        parsed_module.items.iter().map(|item| to_document_symbol(&text, item)).flatten().collect();
    Ok(Some(DocumentSymbolResponse::Nested(symbols)))
}

fn to_document_symbol(text: &str, item: &noirc_frontend::parser::Item) -> Option<DocumentSymbol> {
    match &item.kind {
        ItemKind::Function(function) => Some(noir_function_to_document_symbol(text, function)),
        ItemKind::Struct(struct_) => Some(DocumentSymbol {
            name: struct_.name.to_string(),
            detail: None,
            kind: SymbolKind::STRUCT,
            tags: None,
            deprecated: None,
            range: Default::default(),
            selection_range: Default::default(),
            children: Some(
                struct_
                    .fields
                    .iter()
                    .map(|(ident, type_)| DocumentSymbol {
                        name: ident.to_string(),
                        detail: Some(type_.to_string()),
                        kind: SymbolKind::FIELD,
                        tags: None,
                        deprecated: None,
                        range: Default::default(),
                        selection_range: Default::default(),
                        children: None,
                    })
                    .collect(),
            ),
        }),
        ItemKind::Trait(trait_) => Some(DocumentSymbol {
            name: trait_.name.to_string(),
            detail: None,
            kind: SymbolKind::INTERFACE,
            tags: None,
            deprecated: None,
            range: Default::default(),
            selection_range: Default::default(),
            children: Some(
                trait_
                    .items
                    .iter()
                    .map(|trait_item| match trait_item {
                        TraitItem::Constant { name, typ, default_value } => {
                            (DocumentSymbol {
                                name: name.to_string(),
                                detail: Some(typ.to_string()),
                                kind: SymbolKind::CONSTANT,
                                tags: None,
                                deprecated: None,
                                range: Default::default(),
                                selection_range: Default::default(),
                                children: None,
                            })
                        }
                        TraitItem::Function { name, .. } => {
                            (DocumentSymbol {
                                name: name.to_string(),
                                detail: None,
                                kind: SymbolKind::FUNCTION,
                                tags: None,
                                deprecated: None,
                                range: Default::default(),
                                selection_range: Default::default(),
                                children: None,
                            })
                        }
                        TraitItem::Type { name } => DocumentSymbol {
                            name: name.to_string(),
                            detail: None,
                            kind: SymbolKind::TYPE_PARAMETER,
                            tags: None,
                            deprecated: None,
                            range: Default::default(),
                            selection_range: Default::default(),
                            children: None,
                        },
                    })
                    .collect(),
            ),
        }),
        ItemKind::TraitImpl(impl_) => Some(DocumentSymbol {
            name: impl_.trait_name.to_string(),
            detail: None,
            kind: SymbolKind::OBJECT, // rust-analyzer uses object for impls
            tags: None,
            deprecated: None,
            range: Default::default(),
            selection_range: Default::default(),
            children: Some(
                impl_
                    .items
                    .iter()
                    .map(|item| match item {
                        TraitImplItem::Constant(ident, type_, _) => DocumentSymbol {
                            name: ident.to_string(),
                            detail: Some(type_.to_string()),
                            kind: SymbolKind::CONSTANT,
                            tags: None,
                            deprecated: None,
                            range: Default::default(),
                            selection_range: Default::default(),
                            children: None,
                        },
                        TraitImplItem::Function(function) => {
                            noir_function_to_document_symbol(text, function)
                        }
                        TraitImplItem::Type { name, .. } => DocumentSymbol {
                            name: name.to_string(),
                            detail: None,
                            kind: SymbolKind::TYPE_PARAMETER,
                            tags: None,
                            deprecated: None,
                            range: Default::default(),
                            selection_range: Default::default(),
                            children: None,
                        },
                    })
                    .collect(),
            ),
        }),
        ItemKind::Impl(impl_) => {
            let objname = impl_.object_type.to_string();
            let generics = impl_
                .generics
                .iter()
                .map(|gen| gen.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let name = format!("impl{}<{}> {{}}", objname, generics);
            Some(DocumentSymbol {
                name: name,
                detail: None,
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                range: Default::default(),
                selection_range: Default::default(),
                children: Some(
                    impl_
                        .methods
                        .iter()
                        .map(|item| noir_function_to_document_symbol(text, item))
                        .collect(),
                ),
            })
        }
        ItemKind::Global(global) => {
            if let Pattern::Identifier(ident) = &global.pattern {
                Some(DocumentSymbol {
                    name: ident.to_string(),
                    detail: None,
                    kind: SymbolKind::VARIABLE,
                    tags: None,
                    deprecated: None,
                    range: Default::default(),
                    selection_range: Default::default(),
                    children: None,
                })
            } else {
                None
            }
        }
        ItemKind::ModuleDecl(module) => Some(DocumentSymbol {
            name: module.to_string(),
            detail: None,
            kind: SymbolKind::MODULE,
            tags: None,
            deprecated: None,
            range: Default::default(),
            selection_range: Default::default(),
            children: None,
        }),
        ItemKind::Submodules(submodule) => Some(DocumentSymbol {
            name: submodule.name.to_string(),
            detail: None,
            kind: SymbolKind::MODULE,
            tags: None,
            deprecated: None,
            range: Default::default(),
            selection_range: Default::default(),
            children: Some(
                submodule
                    .contents
                    .items
                    .iter()
                    .filter_map(|item| to_document_symbol(text, item))
                    .collect(),
            ),
        }),
        ItemKind::TypeAlias(_) | ItemKind::Import(_) => None,
    }
}

fn noir_function_to_document_symbol(text: &str, function: &NoirFunction) -> DocumentSymbol {
    DocumentSymbol {
        name: function.name().to_string(),
        detail: None,
        kind: SymbolKind::FUNCTION,
        tags: None,
        deprecated: None,
        range: Default::default(),
        selection_range: Default::default(),
        children: None,
    }
}
