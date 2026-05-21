use std::future::{self, Future};

use async_lsp::ResponseError;
use async_lsp::lsp_types::{
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, Location, Position, SymbolKind,
    TextDocumentPositionParams,
};
use fm::{FileId, FileMap};
use noirc_errors::Span;
use noirc_frontend::ast::TraitBound;
use noirc_frontend::{
    ParsedModule,
    ast::{
        Expression, FunctionReturnType, Ident, LetStatement, NoirFunction, NoirStruct, NoirTrait,
        NoirTraitImpl, TypeImpl, UnresolvedType, UnresolvedTypeData, Visitor,
    },
    parser::ParsedSubModule,
};

use crate::LspState;
use crate::requests::process_request;

pub(crate) fn on_document_symbol_request(
    state: &mut LspState,
    params: DocumentSymbolParams,
) -> impl Future<Output = Result<Option<DocumentSymbolResponse>, ResponseError>> + use<> {
    let text_document_position_params = TextDocumentPositionParams {
        text_document: params.text_document,
        position: Position { line: 0, character: 0 },
    };

    let result = process_request(state, text_document_position_params, |args| {
        let file_id = args.location.file;
        let file = args.files.get_file(file_id).unwrap();
        let source = file.source();
        let (parsed_module, _errors) = noirc_frontend::parse_program(source, file_id);

        let mut collector = DocumentSymbolCollector::new(file_id, args.files);
        let symbols = collector.collect(&parsed_module);
        Some(DocumentSymbolResponse::Nested(symbols))
    });

    future::ready(result)
}

struct DocumentSymbolCollector<'a> {
    file_id: FileId,
    files: &'a FileMap,
    symbols: Vec<DocumentSymbol>,
}

impl<'a> DocumentSymbolCollector<'a> {
    fn new(file_id: FileId, files: &'a FileMap) -> Self {
        Self { file_id, files, symbols: Vec::new() }
    }

    fn collect(&mut self, parsed_module: &ParsedModule) -> Vec<DocumentSymbol> {
        parsed_module.accept(self);

        std::mem::take(&mut self.symbols)
    }

    fn collect_in_type(&mut self, name: &Ident, typ: Option<&UnresolvedType>) {
        if name.is_empty() {
            return;
        }

        let Some(name_location) = self.to_lsp_location(name.span()) else {
            return;
        };

        let span = if let Some(typ) = typ {
            Span::from(name.span().start()..typ.location.span.end())
        } else {
            name.span()
        };

        let Some(location) = self.to_lsp_location(span) else {
            return;
        };

        #[allow(deprecated)]
        self.symbols.push(DocumentSymbol {
            name: name.to_string(),
            detail: None,
            kind: SymbolKind::TYPE_PARAMETER,
            tags: None,
            deprecated: None,
            range: location.range,
            selection_range: name_location.range,
            children: None,
        });
    }

    fn collect_in_constant(
        &mut self,
        name: &Ident,
        typ: &UnresolvedType,
        default_value: Option<&Expression>,
    ) {
        if name.is_empty() {
            return;
        }

        let Some(name_location) = self.to_lsp_location(name.span()) else {
            return;
        };

        let mut span = name.span();

        // If there's a type span, extend the span to include it
        span = Span::from(span.start()..typ.location.span.end());

        // If there's a default value, extend the span to include it
        if let Some(default_value) = default_value {
            span = Span::from(span.start()..default_value.location.span.end());
        }

        let Some(location) = self.to_lsp_location(span) else {
            return;
        };

        #[allow(deprecated)]
        self.symbols.push(DocumentSymbol {
            name: name.to_string(),
            detail: None,
            kind: SymbolKind::CONSTANT,
            tags: None,
            deprecated: None,
            range: location.range,
            selection_range: name_location.range,
            children: None,
        });
    }

    fn to_lsp_location(&self, span: Span) -> Option<Location> {
        super::to_lsp_location(self.files, self.file_id, span)
    }
}

impl Visitor for DocumentSymbolCollector<'_> {
    fn visit_noir_function(&mut self, noir_function: &NoirFunction, span: Span) -> bool {
        if noir_function.def.name.is_empty() {
            return false;
        }

        let Some(location) = self.to_lsp_location(span) else {
            return false;
        };

        let Some(selection_location) = self.to_lsp_location(noir_function.name_ident().span())
        else {
            return false;
        };

        #[allow(deprecated)]
        self.symbols.push(DocumentSymbol {
            name: noir_function.name().to_string(),
            detail: Some(noir_function.def.signature()),
            kind: SymbolKind::FUNCTION,
            tags: None,
            deprecated: None,
            range: location.range,
            selection_range: selection_location.range,
            children: None,
        });

        false
    }

    fn visit_noir_struct(&mut self, noir_struct: &NoirStruct, span: Span) -> bool {
        if noir_struct.name.is_empty() {
            return false;
        }

        let Some(location) = self.to_lsp_location(span) else {
            return false;
        };

        let Some(selection_location) = self.to_lsp_location(noir_struct.name.span()) else {
            return false;
        };

        let mut children = Vec::new();
        for field in &noir_struct.fields {
            let field_name = &field.item.name;
            let typ = &field.item.typ;
            let span = Span::from(field_name.span().start()..typ.location.span.end());

            let Some(field_location) = self.to_lsp_location(span) else {
                continue;
            };

            let Some(field_name_location) = self.to_lsp_location(field_name.span()) else {
                continue;
            };

            #[allow(deprecated)]
            children.push(DocumentSymbol {
                name: field_name.to_string(),
                detail: None,
                kind: SymbolKind::FIELD,
                tags: None,
                deprecated: None,
                range: field_location.range,
                selection_range: field_name_location.range,
                children: None,
            });
        }

        #[allow(deprecated)]
        self.symbols.push(DocumentSymbol {
            name: noir_struct.name.to_string(),
            detail: None,
            kind: SymbolKind::STRUCT,
            tags: None,
            deprecated: None,
            range: location.range,
            selection_range: selection_location.range,
            children: Some(children),
        });

        false
    }

    fn visit_noir_trait(&mut self, noir_trait: &NoirTrait, span: Span) -> bool {
        if noir_trait.name.is_empty() {
            return false;
        }

        let Some(location) = self.to_lsp_location(span) else {
            return false;
        };

        let Some(selection_location) = self.to_lsp_location(noir_trait.name.span()) else {
            return false;
        };

        let old_symbols = std::mem::take(&mut self.symbols);
        self.symbols = Vec::new();

        for item in &noir_trait.items {
            item.item.accept(self);
        }

        let children = std::mem::take(&mut self.symbols);
        self.symbols = old_symbols;

        #[allow(deprecated)]
        self.symbols.push(DocumentSymbol {
            name: noir_trait.name.to_string(),
            detail: None,
            kind: SymbolKind::INTERFACE,
            tags: None,
            deprecated: None,
            range: location.range,
            selection_range: selection_location.range,
            children: Some(children),
        });

        false
    }

    fn visit_trait_item_function(
        &mut self,
        name: &Ident,
        _generics: &noirc_frontend::ast::UnresolvedGenerics,
        parameters: &[(Ident, UnresolvedType)],
        return_type: &FunctionReturnType,
        _where_clause: &[noirc_frontend::ast::UnresolvedTraitConstraint],
        body: &Option<noirc_frontend::ast::BlockExpression>,
    ) -> bool {
        if name.is_empty() {
            return false;
        }

        let Some(name_location) = self.to_lsp_location(name.span()) else {
            return false;
        };

        let mut span = name.span();

        // If there are parameters, extend the span to include the last parameter.
        if let Some((param_name, _param_type)) = parameters.last() {
            span = Span::from(span.start()..param_name.span().end());
        }

        // If there's a return type, extend the span to include it
        match return_type {
            FunctionReturnType::Default(return_type_location) => {
                span = Span::from(span.start()..return_type_location.span.end());
            }
            FunctionReturnType::Ty(typ) => {
                span = Span::from(span.start()..typ.location.span.end());
            }
        }

        // If there's a body, extend the span to include it
        if let Some(body) = body
            && let Some(statement) = body.statements.last()
        {
            span = Span::from(span.start()..statement.location.span.end());
        }

        let Some(location) = self.to_lsp_location(span) else {
            return false;
        };

        #[allow(deprecated)]
        self.symbols.push(DocumentSymbol {
            name: name.to_string(),
            detail: None,
            kind: SymbolKind::METHOD,
            tags: None,
            deprecated: None,
            range: location.range,
            selection_range: name_location.range,
            children: None,
        });

        false
    }

    fn visit_trait_item_constant(&mut self, name: &Ident, typ: Option<&UnresolvedType>) -> bool {
        if name.is_empty() {
            return false;
        }

        if let Some(typ) = typ {
            self.collect_in_constant(name, typ, None);
        }
        false
    }

    fn visit_trait_item_type(&mut self, name: &Ident, _bounds: &[TraitBound]) -> bool {
        self.collect_in_type(name, None);
        false
    }

    fn visit_noir_trait_impl(&mut self, noir_trait_impl: &NoirTraitImpl, span: Span) -> bool {
        let Some(location) = self.to_lsp_location(span) else {
            return false;
        };

        let name_location =
            if let UnresolvedTypeData::Named(trait_name, _, _) = &noir_trait_impl.r#trait.typ {
                trait_name.location
            } else {
                noir_trait_impl.r#trait.location
            };

        let Some(name_location) = self.to_lsp_location(name_location.span) else {
            return false;
        };

        let trait_name = noir_trait_impl.r#trait.to_string();

        let old_symbols = std::mem::take(&mut self.symbols);
        self.symbols = Vec::new();

        for trait_impl_item in &noir_trait_impl.items {
            trait_impl_item.item.accept(self);
        }

        let children = std::mem::take(&mut self.symbols);
        self.symbols = old_symbols;

        #[allow(deprecated)]
        self.symbols.push(DocumentSymbol {
            name: format!("impl {} for {}", trait_name, noir_trait_impl.object_type),
            detail: None,
            kind: SymbolKind::NAMESPACE,
            tags: None,
            deprecated: None,
            range: location.range,
            selection_range: name_location.range,
            children: Some(children),
        });

        false
    }

    fn visit_trait_impl_item_constant(
        &mut self,
        name: &Ident,
        typ: Option<&UnresolvedType>,
        default_value: &Expression,
        _span: Span,
    ) -> bool {
        if let Some(typ) = typ {
            self.collect_in_constant(name, typ, Some(default_value));
        }
        false
    }

    fn visit_trait_impl_item_type(
        &mut self,
        name: &Ident,
        alias: Option<&UnresolvedType>,
        _span: Span,
    ) -> bool {
        if let Some(alias) = alias {
            self.collect_in_type(name, Some(alias));
        }
        false
    }

    fn visit_type_impl(&mut self, type_impl: &TypeImpl, span: Span) -> bool {
        let Some(location) = self.to_lsp_location(span) else {
            return false;
        };

        let name = type_impl.object_type.typ.to_string();
        if name.is_empty() {
            return false;
        }

        let Some(name_location) = self.to_lsp_location(type_impl.object_type.location.span) else {
            return false;
        };

        let old_symbols = std::mem::take(&mut self.symbols);
        self.symbols = Vec::new();

        for (noir_function, noir_function_location) in &type_impl.methods {
            noir_function.item.accept(noir_function_location.span, self);
        }

        let children = std::mem::take(&mut self.symbols);
        self.symbols = old_symbols;

        #[allow(deprecated)]
        self.symbols.push(DocumentSymbol {
            name,
            detail: None,
            kind: SymbolKind::NAMESPACE,
            tags: None,
            deprecated: None,
            range: location.range,
            selection_range: name_location.range,
            children: Some(children),
        });

        false
    }

    fn visit_parsed_submodule(&mut self, parsed_sub_module: &ParsedSubModule, span: Span) -> bool {
        if parsed_sub_module.name.is_empty() {
            return false;
        }

        let Some(name_location) = self.to_lsp_location(parsed_sub_module.name.span()) else {
            return false;
        };

        let Some(location) = self.to_lsp_location(span) else {
            return false;
        };

        let old_symbols = std::mem::take(&mut self.symbols);
        self.symbols = Vec::new();

        for item in &parsed_sub_module.contents.items {
            item.accept(self);
        }

        let children = std::mem::take(&mut self.symbols);
        self.symbols = old_symbols;

        #[allow(deprecated)]
        self.symbols.push(DocumentSymbol {
            name: parsed_sub_module.name.to_string(),
            detail: None,
            kind: SymbolKind::MODULE,
            tags: None,
            deprecated: None,
            range: location.range,
            selection_range: name_location.range,
            children: Some(children),
        });

        false
    }

    fn visit_global(&mut self, global: &LetStatement, span: Span) -> bool {
        let name = global.pattern.to_string();
        if name.is_empty() {
            return false;
        }

        let Some(name_location) = self.to_lsp_location(global.pattern.span()) else {
            return false;
        };

        let Some(location) = self.to_lsp_location(span) else {
            return false;
        };

        #[allow(deprecated)]
        self.symbols.push(DocumentSymbol {
            name,
            detail: None,
            kind: SymbolKind::CONSTANT,
            tags: None,
            deprecated: None,
            range: location.range,
            selection_range: name_location.range,
            children: None,
        });

        false
    }
}

#[cfg(test)]
mod document_symbol_tests {
    use crate::test_utils;

    use super::*;
    use async_lsp::lsp_types::{
        PartialResultParams, Range, SymbolKind, TextDocumentIdentifier, WorkDoneProgressParams,
    };
    use tokio::test;

    async fn get_document_symbols(src: &str) -> Vec<DocumentSymbol> {
        let (mut state, noir_text_document) =
            test_utils::init_lsp_server_with_inline_source("document_symbol", "src/main.nr", src)
                .await;

        let response = on_document_symbol_request(
            &mut state,
            DocumentSymbolParams {
                text_document: TextDocumentIdentifier { uri: noir_text_document },
                work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
                partial_result_params: PartialResultParams { partial_result_token: None },
            },
        )
        .await
        .expect("Could not execute on_document_symbol_request")
        .unwrap();

        let DocumentSymbolResponse::Nested(symbols) = response else {
            panic!("Expected response to be nested");
        };

        symbols
    }

    #[test]
    async fn test_document_symbol_for_function() {
        let src = r#"fn foo(_x: i32) {
    let _ = 1;
}
"#;
        let symbols = get_document_symbols(src).await;

        assert_eq!(
            symbols,
            vec![
                #[allow(deprecated)]
                DocumentSymbol {
                    name: "foo".to_string(),
                    detail: Some("fn foo(_x: i32)".to_string()),
                    kind: SymbolKind::FUNCTION,
                    tags: None,
                    deprecated: None,
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 2, character: 1 },
                    },
                    selection_range: Range {
                        start: Position { line: 0, character: 3 },
                        end: Position { line: 0, character: 6 },
                    },
                    children: None,
                },
            ]
        );
    }

    #[test]
    async fn test_document_symbol_for_struct_with_field() {
        let src = r#"struct SomeStruct {
    field: i32,
}
"#;
        let symbols = get_document_symbols(src).await;

        assert_eq!(
            symbols,
            vec![
                #[allow(deprecated)]
                DocumentSymbol {
                    name: "SomeStruct".to_string(),
                    detail: None,
                    kind: SymbolKind::STRUCT,
                    tags: None,
                    deprecated: None,
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 2, character: 1 },
                    },
                    selection_range: Range {
                        start: Position { line: 0, character: 7 },
                        end: Position { line: 0, character: 17 },
                    },
                    children: Some(vec![
                        #[allow(deprecated)]
                        DocumentSymbol {
                            name: "field".to_string(),
                            detail: None,
                            kind: SymbolKind::FIELD,
                            tags: None,
                            deprecated: None,
                            range: Range {
                                start: Position { line: 1, character: 4 },
                                end: Position { line: 1, character: 14 },
                            },
                            selection_range: Range {
                                start: Position { line: 1, character: 4 },
                                end: Position { line: 1, character: 9 },
                            },
                            children: None,
                        },
                    ],),
                },
            ]
        );
    }

    #[test]
    async fn test_document_symbol_for_inherent_impl() {
        let src = r#"struct SomeStruct {}

impl SomeStruct {
    fn new() -> SomeStruct {
        SomeStruct {}
    }
}
"#;
        let symbols = get_document_symbols(src).await;

        let impl_symbol = &symbols[1];
        assert_eq!(impl_symbol.name, "SomeStruct");
        assert_eq!(impl_symbol.kind, SymbolKind::NAMESPACE);
        assert_eq!(
            impl_symbol.range,
            Range {
                start: Position { line: 2, character: 0 },
                end: Position { line: 6, character: 1 },
            }
        );
        assert_eq!(
            impl_symbol.selection_range,
            Range {
                start: Position { line: 2, character: 5 },
                end: Position { line: 2, character: 15 },
            }
        );

        let children = impl_symbol.children.as_ref().expect("Expected children");
        assert_eq!(children.len(), 1);
        let method = &children[0];
        assert_eq!(method.name, "new");
        assert_eq!(method.detail.as_deref(), Some("fn new() -> SomeStruct"));
        assert_eq!(method.kind, SymbolKind::FUNCTION);
        assert_eq!(
            method.range,
            Range {
                start: Position { line: 3, character: 4 },
                end: Position { line: 5, character: 5 },
            }
        );
        assert_eq!(
            method.selection_range,
            Range {
                start: Position { line: 3, character: 7 },
                end: Position { line: 3, character: 10 },
            }
        );
    }

    #[test]
    async fn test_document_symbol_for_trait() {
        let src = r#"trait SomeTrait<U> {
    fn some_method(x: U);
}
"#;
        let symbols = get_document_symbols(src).await;

        assert_eq!(
            symbols,
            vec![
                #[allow(deprecated)]
                DocumentSymbol {
                    name: "SomeTrait".to_string(),
                    detail: None,
                    kind: SymbolKind::INTERFACE,
                    tags: None,
                    deprecated: None,
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 2, character: 1 },
                    },
                    selection_range: Range {
                        start: Position { line: 0, character: 6 },
                        end: Position { line: 0, character: 15 },
                    },
                    children: Some(vec![
                        #[allow(deprecated)]
                        DocumentSymbol {
                            name: "some_method".to_string(),
                            detail: None,
                            kind: SymbolKind::METHOD,
                            tags: None,
                            deprecated: None,
                            range: Range {
                                start: Position { line: 1, character: 7 },
                                end: Position { line: 1, character: 25 },
                            },
                            selection_range: Range {
                                start: Position { line: 1, character: 7 },
                                end: Position { line: 1, character: 18 },
                            },
                            children: None,
                        },
                    ],),
                },
            ]
        );
    }

    #[test]
    async fn test_document_symbol_for_trait_impl() {
        let src = r#"struct SomeStruct {}

trait SomeTrait<U> {
    fn some_method(x: U);
}

impl SomeTrait<i32> for SomeStruct {
    fn some_method(_x: i32) {
    }
}
"#;
        let symbols = get_document_symbols(src).await;

        // [struct SomeStruct, trait SomeTrait, impl SomeTrait<i32> for SomeStruct]
        assert_eq!(symbols.len(), 3);
        let impl_symbol = &symbols[2];
        assert_eq!(impl_symbol.name, "impl SomeTrait<i32> for SomeStruct");
        assert_eq!(impl_symbol.kind, SymbolKind::NAMESPACE);
        assert_eq!(
            impl_symbol.range,
            Range {
                start: Position { line: 6, character: 0 },
                end: Position { line: 9, character: 1 },
            }
        );
        assert_eq!(
            impl_symbol.selection_range,
            Range {
                start: Position { line: 6, character: 5 },
                end: Position { line: 6, character: 14 },
            }
        );

        let children = impl_symbol.children.as_ref().expect("Expected children");
        assert_eq!(children.len(), 1);
        let method = &children[0];
        assert_eq!(method.name, "some_method");
        assert_eq!(method.detail.as_deref(), Some("fn some_method(_x: i32)"));
        assert_eq!(method.kind, SymbolKind::FUNCTION);
    }

    #[test]
    async fn test_document_symbol_for_module_with_global() {
        let src = r#"mod submodule {
    global SOME_GLOBAL = 1;
}
"#;
        let symbols = get_document_symbols(src).await;

        assert_eq!(
            symbols,
            vec![
                #[allow(deprecated)]
                DocumentSymbol {
                    name: "submodule".to_string(),
                    detail: None,
                    kind: SymbolKind::MODULE,
                    tags: None,
                    deprecated: None,
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 2, character: 1 },
                    },
                    selection_range: Range {
                        start: Position { line: 0, character: 4 },
                        end: Position { line: 0, character: 13 },
                    },
                    children: Some(vec![
                        #[allow(deprecated)]
                        DocumentSymbol {
                            name: "SOME_GLOBAL".to_string(),
                            detail: None,
                            kind: SymbolKind::CONSTANT,
                            tags: None,
                            deprecated: None,
                            range: Range {
                                start: Position { line: 1, character: 4 },
                                end: Position { line: 1, character: 27 },
                            },
                            selection_range: Range {
                                start: Position { line: 1, character: 11 },
                                end: Position { line: 1, character: 22 },
                            },
                            children: None,
                        },
                    ]),
                },
            ]
        );
    }

    #[test]
    async fn test_document_symbol_for_primitive_impl() {
        let src = "impl i32 {}\n";
        let symbols = get_document_symbols(src).await;

        assert_eq!(
            symbols,
            vec![
                #[allow(deprecated)]
                DocumentSymbol {
                    name: "i32".to_string(),
                    detail: None,
                    kind: SymbolKind::NAMESPACE,
                    tags: None,
                    deprecated: None,
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 11 },
                    },
                    selection_range: Range {
                        start: Position { line: 0, character: 5 },
                        end: Position { line: 0, character: 8 },
                    },
                    children: Some(Vec::new()),
                },
            ]
        );
    }

    #[test]
    async fn test_function_with_just_open_parentheses() {
        let src = "fn main(\n";
        let mut symbols = get_document_symbols(src).await;
        assert_eq!(symbols.len(), 1);
        let symbol = symbols.remove(0);
        assert_eq!(
            symbol.range,
            Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 1, character: 0 },
            }
        );
        assert_eq!(
            symbol.selection_range,
            Range {
                start: Position { line: 0, character: 3 },
                end: Position { line: 0, character: 7 },
            }
        );
    }
}
