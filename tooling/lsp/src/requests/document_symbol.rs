use std::future::{self, Future};

use async_lsp::ResponseError;
use fm::{FileId, FileMap, PathString};
use lsp_types::{
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, Location, Position, SymbolKind,
    TextDocumentPositionParams,
};
use noirc_errors::Span;
use noirc_frontend::{
    ast::{
        Expression, FunctionReturnType, Ident, LetStatement, NoirFunction, NoirStruct, NoirTrait,
        NoirTraitImpl, TypeImpl, UnresolvedType, Visitor,
    },
    parser::ParsedSubModule,
    ParsedModule,
};

use crate::LspState;

use super::process_request;

pub(crate) fn on_document_symbol_request(
    state: &mut LspState,
    params: DocumentSymbolParams,
) -> impl Future<Output = Result<Option<DocumentSymbolResponse>, ResponseError>> {
    let Ok(file_path) = params.text_document.uri.to_file_path() else {
        return future::ready(Ok(None));
    };

    let text_document_position_params = TextDocumentPositionParams {
        text_document: params.text_document.clone(),
        position: Position { line: 0, character: 0 },
    };

    let result = process_request(state, text_document_position_params, |args| {
        args.files.get_file_id(&PathString::from_path(file_path)).map(|file_id| {
            let file = args.files.get_file(file_id).unwrap();
            let source = file.source();
            let (parsed_module, _errors) = noirc_frontend::parse_program(source);

            let mut collector = DocumentSymbolCollector::new(file_id, args.files);
            let symbols = collector.collect(&parsed_module);
            DocumentSymbolResponse::Nested(symbols)
        })
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
        if name.0.contents.is_empty() {
            return;
        }

        let Some(name_location) = self.to_lsp_location(name.span()) else {
            return;
        };

        let span = if let Some(typ) = typ {
            Span::from(name.span().start()..typ.span.end())
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
        if name.0.contents.is_empty() {
            return;
        }

        let Some(name_location) = self.to_lsp_location(name.span()) else {
            return;
        };

        let mut span = name.span();

        // If there's a type span, extend the span to include it
        span = Span::from(span.start()..typ.span.end());

        // If there's a default value, extend the span to include it
        if let Some(default_value) = default_value {
            span = Span::from(span.start()..default_value.span.end());
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

impl<'a> Visitor for DocumentSymbolCollector<'a> {
    fn visit_noir_function(&mut self, noir_function: &NoirFunction, span: Span) -> bool {
        if noir_function.def.name.0.contents.is_empty() {
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
        if noir_struct.name.0.contents.is_empty() {
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
            let span = Span::from(field_name.span().start()..typ.span.end());

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
        if noir_trait.name.0.contents.is_empty() {
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
        if name.0.contents.is_empty() {
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
            FunctionReturnType::Default(return_type_span) => {
                span = Span::from(span.start()..return_type_span.end());
            }
            FunctionReturnType::Ty(typ) => {
                span = Span::from(span.start()..typ.span.end());
            }
        }

        // If there's a body, extend the span to include it
        if let Some(body) = body {
            if let Some(statement) = body.statements.last() {
                span = Span::from(span.start()..statement.span.end());
            }
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

    fn visit_trait_item_constant(
        &mut self,
        name: &Ident,
        typ: &UnresolvedType,
        default_value: &Option<Expression>,
    ) -> bool {
        if name.0.contents.is_empty() {
            return false;
        }

        self.collect_in_constant(name, typ, default_value.as_ref());
        false
    }

    fn visit_trait_item_type(&mut self, name: &Ident) {
        self.collect_in_type(name, None);
    }

    fn visit_noir_trait_impl(&mut self, noir_trait_impl: &NoirTraitImpl, span: Span) -> bool {
        let Some(location) = self.to_lsp_location(span) else {
            return false;
        };

        let Some(name_location) = self.to_lsp_location(noir_trait_impl.trait_name.span) else {
            return false;
        };

        let mut trait_name = String::new();
        trait_name.push_str(&noir_trait_impl.trait_name.to_string());
        if !noir_trait_impl.trait_generics.is_empty() {
            trait_name.push('<');
            for (index, generic) in noir_trait_impl.trait_generics.ordered_args.iter().enumerate() {
                if index > 0 {
                    trait_name.push_str(", ");
                }
                trait_name.push_str(&generic.to_string());
            }
            for (index, (name, generic)) in
                noir_trait_impl.trait_generics.named_args.iter().enumerate()
            {
                if index > 0 {
                    trait_name.push_str(", ");
                }
                trait_name.push_str(&name.0.contents);
                trait_name.push_str(" = ");
                trait_name.push_str(&generic.to_string());
            }
            trait_name.push('>');
        }

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
        typ: &UnresolvedType,
        default_value: &Expression,
        _span: Span,
    ) -> bool {
        self.collect_in_constant(name, typ, Some(default_value));
        false
    }

    fn visit_trait_impl_item_type(
        &mut self,
        name: &Ident,
        alias: &UnresolvedType,
        _span: Span,
    ) -> bool {
        self.collect_in_type(name, Some(alias));
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

        let Some(name_location) = self.to_lsp_location(type_impl.object_type.span) else {
            return false;
        };

        let old_symbols = std::mem::take(&mut self.symbols);
        self.symbols = Vec::new();

        for (noir_function, noir_function_span) in &type_impl.methods {
            noir_function.item.accept(*noir_function_span, self);
        }

        let children = std::mem::take(&mut self.symbols);
        self.symbols = old_symbols;

        #[allow(deprecated)]
        self.symbols.push(DocumentSymbol {
            name: name.to_string(),
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
        if parsed_sub_module.name.0.contents.is_empty() {
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
    use lsp_types::{
        PartialResultParams, Range, SymbolKind, TextDocumentIdentifier, WorkDoneProgressParams,
    };
    use tokio::test;

    #[test]
    async fn test_document_symbol() {
        let (mut state, noir_text_document) = test_utils::init_lsp_server("document_symbol").await;

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
                #[allow(deprecated)]
                DocumentSymbol {
                    name: "SomeStruct".to_string(),
                    detail: None,
                    kind: SymbolKind::STRUCT,
                    tags: None,
                    deprecated: None,
                    range: Range {
                        start: Position { line: 4, character: 0 },
                        end: Position { line: 6, character: 1 },
                    },
                    selection_range: Range {
                        start: Position { line: 4, character: 7 },
                        end: Position { line: 4, character: 17 },
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
                                start: Position { line: 5, character: 4 },
                                end: Position { line: 5, character: 14 },
                            },
                            selection_range: Range {
                                start: Position { line: 5, character: 4 },
                                end: Position { line: 5, character: 9 },
                            },
                            children: None,
                        },
                    ],),
                },
                #[allow(deprecated)]
                DocumentSymbol {
                    name: "SomeStruct".to_string(),
                    detail: None,
                    kind: SymbolKind::NAMESPACE,
                    tags: None,
                    deprecated: None,
                    range: Range {
                        start: Position { line: 8, character: 0 },
                        end: Position { line: 12, character: 1 },
                    },
                    selection_range: Range {
                        start: Position { line: 8, character: 5 },
                        end: Position { line: 8, character: 15 },
                    },
                    children: Some(vec![
                        #[allow(deprecated)]
                        DocumentSymbol {
                            name: "new".to_string(),
                            detail: Some("fn new() -> SomeStruct".to_string()),
                            kind: SymbolKind::FUNCTION,
                            tags: None,
                            deprecated: None,
                            range: Range {
                                start: Position { line: 9, character: 4 },
                                end: Position { line: 11, character: 5 },
                            },
                            selection_range: Range {
                                start: Position { line: 9, character: 7 },
                                end: Position { line: 9, character: 10 },
                            },
                            children: None,
                        },
                    ],),
                },
                #[allow(deprecated)]
                DocumentSymbol {
                    name: "SomeTrait".to_string(),
                    detail: None,
                    kind: SymbolKind::INTERFACE,
                    tags: None,
                    deprecated: None,
                    range: Range {
                        start: Position { line: 14, character: 0 },
                        end: Position { line: 16, character: 1 },
                    },
                    selection_range: Range {
                        start: Position { line: 14, character: 6 },
                        end: Position { line: 14, character: 15 },
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
                                start: Position { line: 15, character: 7 },
                                end: Position { line: 15, character: 24 },
                            },
                            selection_range: Range {
                                start: Position { line: 15, character: 7 },
                                end: Position { line: 15, character: 18 },
                            },
                            children: None,
                        },
                    ],),
                },
                #[allow(deprecated)]
                DocumentSymbol {
                    name: "impl SomeTrait<i32> for SomeStruct".to_string(),
                    detail: None,
                    kind: SymbolKind::NAMESPACE,
                    tags: None,
                    deprecated: None,
                    range: Range {
                        start: Position { line: 18, character: 0 },
                        end: Position { line: 21, character: 1 },
                    },
                    selection_range: Range {
                        start: Position { line: 18, character: 5 },
                        end: Position { line: 18, character: 14 },
                    },
                    children: Some(vec![
                        #[allow(deprecated)]
                        DocumentSymbol {
                            name: "some_method".to_string(),
                            detail: Some("fn some_method(_x: i32)".to_string()),
                            kind: SymbolKind::FUNCTION,
                            tags: None,
                            deprecated: None,
                            range: Range {
                                start: Position { line: 19, character: 4 },
                                end: Position { line: 20, character: 5 },
                            },
                            selection_range: Range {
                                start: Position { line: 19, character: 7 },
                                end: Position { line: 19, character: 18 },
                            },
                            children: None,
                        },
                    ],),
                },
                #[allow(deprecated)]
                DocumentSymbol {
                    name: "submodule".to_string(),
                    detail: None,
                    kind: SymbolKind::MODULE,
                    tags: None,
                    deprecated: None,
                    range: Range {
                        start: Position { line: 23, character: 0 },
                        end: Position { line: 25, character: 1 },
                    },
                    selection_range: Range {
                        start: Position { line: 23, character: 4 },
                        end: Position { line: 23, character: 13 },
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
                                start: Position { line: 24, character: 4 },
                                end: Position { line: 24, character: 27 }
                            },
                            selection_range: Range {
                                start: Position { line: 24, character: 11 },
                                end: Position { line: 24, character: 22 }
                            },
                            children: None
                        }
                    ]),
                },
                #[allow(deprecated)]
                DocumentSymbol {
                    name: "i32".to_string(),
                    detail: None,
                    kind: SymbolKind::NAMESPACE,
                    tags: None,
                    deprecated: None,
                    range: Range {
                        start: Position { line: 27, character: 0 },
                        end: Position { line: 27, character: 11 }
                    },
                    selection_range: Range {
                        start: Position { line: 27, character: 5 },
                        end: Position { line: 27, character: 8 }
                    },
                    children: Some(Vec::new())
                }
            ]
        );
    }
}
