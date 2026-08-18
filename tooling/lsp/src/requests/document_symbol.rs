use async_lsp::ResponseError;
use async_lsp::lsp_types::{
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, Location, SymbolKind,
};
use std::collections::HashMap;
use std::path::PathBuf;

use fm::{FileId, FileMap, PathString};
use noirc_errors::Span;
use noirc_frontend::ast::TraitBound;
use noirc_frontend::{
    ParsedModule,
    ast::{
        Expression, FunctionReturnType, Ident, LetStatement, ModuleDeclaration, NoirEnumeration,
        NoirFunction, NoirStruct, NoirTrait, NoirTraitImpl, TypeAlias, TypeImpl, UnresolvedType,
        UnresolvedTypeData, Visitor,
    },
    parser::ParsedSubModule,
};

/// Like formatting, this request is parse-only: it takes the open documents' current texts
/// instead of `LspState`, so the main loop answers it directly from its text mirror instead
/// of queueing it behind type-checking.
pub(crate) fn on_document_symbol_request(
    input_files: &HashMap<String, String>,
    params: DocumentSymbolParams,
) -> Result<Option<DocumentSymbolResponse>, ResponseError> {
    let uri = params.text_document.uri;
    let Some(source) = input_files.get(&uri.to_string()) else {
        return Ok(None);
    };

    let mut files = FileMap::default();
    let file_id = files.add_file(PathString::from_path(PathBuf::from(uri.path())), source.clone());
    let (parsed_module, _errors) = noirc_frontend::parse_program(source, file_id);

    let mut collector = DocumentSymbolCollector::new(file_id, &files);
    let symbols = collector.collect(&parsed_module);
    Ok(Some(DocumentSymbolResponse::Nested(symbols)))
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

    fn visit_noir_enum(&mut self, noir_enum: &NoirEnumeration, span: Span) -> bool {
        if noir_enum.name.is_empty() {
            return false;
        }

        let Some(location) = self.to_lsp_location(span) else {
            return false;
        };

        let Some(selection_location) = self.to_lsp_location(noir_enum.name.span()) else {
            return false;
        };

        let mut children = Vec::new();
        for variant in &noir_enum.variants {
            let variant_name = &variant.item.name;

            let mut span = variant_name.span();

            // If there are parameters, extend the span to include the last parameter type.
            if let Some(parameters) = &variant.item.parameters
                && let Some(typ) = parameters.last()
            {
                span = Span::from(span.start()..typ.location.span.end());
            }

            let Some(variant_location) = self.to_lsp_location(span) else {
                continue;
            };

            let Some(variant_name_location) = self.to_lsp_location(variant_name.span()) else {
                continue;
            };

            #[allow(deprecated)]
            children.push(DocumentSymbol {
                name: variant_name.to_string(),
                detail: None,
                kind: SymbolKind::ENUM_MEMBER,
                tags: None,
                deprecated: None,
                range: variant_location.range,
                selection_range: variant_name_location.range,
                children: None,
            });
        }

        #[allow(deprecated)]
        self.symbols.push(DocumentSymbol {
            name: noir_enum.name.to_string(),
            detail: None,
            kind: SymbolKind::ENUM,
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

    fn visit_noir_type_alias(&mut self, type_alias: &TypeAlias, span: Span) -> bool {
        if type_alias.name.is_empty() {
            return false;
        }

        let Some(location) = self.to_lsp_location(span) else {
            return false;
        };

        let Some(selection_location) = self.to_lsp_location(type_alias.name.span()) else {
            return false;
        };

        #[allow(deprecated)]
        self.symbols.push(DocumentSymbol {
            name: type_alias.name.to_string(),
            detail: None,
            kind: SymbolKind::TYPE_PARAMETER,
            tags: None,
            deprecated: None,
            range: location.range,
            selection_range: selection_location.range,
            children: None,
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

    fn visit_module_declaration(&mut self, module_declaration: &ModuleDeclaration, span: Span) {
        if module_declaration.ident.is_empty() {
            return;
        }

        let Some(name_location) = self.to_lsp_location(module_declaration.ident.span()) else {
            return;
        };

        let Some(location) = self.to_lsp_location(span) else {
            return;
        };

        #[allow(deprecated)]
        self.symbols.push(DocumentSymbol {
            name: module_declaration.ident.to_string(),
            detail: None,
            kind: SymbolKind::MODULE,
            tags: None,
            deprecated: None,
            range: location.range,
            selection_range: name_location.range,
            children: None,
        });
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
        PartialResultParams, SymbolKind, TextDocumentIdentifier, Url, WorkDoneProgressParams,
    };

    fn get_document_symbols(src: &str) -> Vec<DocumentSymbol> {
        let uri = Url::parse("file:///main.nr").unwrap();
        let input_files = HashMap::from([(uri.to_string(), src.to_string())]);

        let response = on_document_symbol_request(
            &input_files,
            DocumentSymbolParams {
                text_document: TextDocumentIdentifier { uri },
                work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
                partial_result_params: PartialResultParams { partial_result_token: None },
            },
        )
        .expect("Could not execute on_document_symbol_request")
        .unwrap();

        let DocumentSymbolResponse::Nested(symbols) = response else {
            panic!("Expected response to be nested");
        };

        symbols
    }

    #[test]
    fn test_document_symbol_for_function() {
        let src = r#"fn foo(_x: i32) {
    let _ = 1;
}
"#;
        let symbols = get_document_symbols(src);

        assert_eq!(symbols.len(), 1);
        let symbol = &symbols[0];
        assert_eq!(symbol.name, "foo");
        assert_eq!(symbol.detail.as_deref(), Some("fn foo(_x: i32)"));
        assert_eq!(symbol.kind, SymbolKind::FUNCTION);
        assert!(symbol.children.is_none());
        // `range` covers the whole function (signature + body).
        assert_eq!(test_utils::text_at(src, symbol.range), "fn foo(_x: i32) {\n    let _ = 1;\n}");
        // `selection_range` covers just the function name.
        assert_eq!(test_utils::text_at(src, symbol.selection_range), "foo");
    }

    #[test]
    fn test_document_symbol_for_struct_with_field() {
        let src = r#"struct SomeStruct {
    field: i32,
}
"#;
        let symbols = get_document_symbols(src);

        assert_eq!(symbols.len(), 1);
        let symbol = &symbols[0];
        assert_eq!(symbol.name, "SomeStruct");
        assert_eq!(symbol.kind, SymbolKind::STRUCT);
        assert_eq!(
            test_utils::text_at(src, symbol.range),
            "struct SomeStruct {\n    field: i32,\n}"
        );
        assert_eq!(test_utils::text_at(src, symbol.selection_range), "SomeStruct");

        let children = symbol.children.as_ref().expect("Expected children");
        assert_eq!(children.len(), 1);
        let field = &children[0];
        assert_eq!(field.name, "field");
        assert_eq!(field.kind, SymbolKind::FIELD);
        assert_eq!(test_utils::text_at(src, field.range), "field: i32");
        assert_eq!(test_utils::text_at(src, field.selection_range), "field");
    }

    #[test]
    fn test_document_symbol_for_inherent_impl() {
        let src = r#"struct SomeStruct {}

impl SomeStruct {
    fn new() -> SomeStruct {
        SomeStruct {}
    }
}
"#;
        let symbols = get_document_symbols(src);

        let impl_symbol = &symbols[1];
        assert_eq!(impl_symbol.name, "SomeStruct");
        assert_eq!(impl_symbol.kind, SymbolKind::NAMESPACE);
        assert_eq!(
            test_utils::text_at(src, impl_symbol.range),
            "impl SomeStruct {\n    fn new() -> SomeStruct {\n        SomeStruct {}\n    }\n}"
        );
        assert_eq!(test_utils::text_at(src, impl_symbol.selection_range), "SomeStruct");

        let children = impl_symbol.children.as_ref().expect("Expected children");
        assert_eq!(children.len(), 1);
        let method = &children[0];
        assert_eq!(method.name, "new");
        assert_eq!(method.detail.as_deref(), Some("fn new() -> SomeStruct"));
        assert_eq!(method.kind, SymbolKind::FUNCTION);
        assert_eq!(
            test_utils::text_at(src, method.range),
            "fn new() -> SomeStruct {\n        SomeStruct {}\n    }"
        );
        assert_eq!(test_utils::text_at(src, method.selection_range), "new");
    }

    #[test]
    fn test_document_symbol_for_trait() {
        let src = r#"trait SomeTrait<U> {
    fn some_method(x: U);
}
"#;
        let symbols = get_document_symbols(src);

        assert_eq!(symbols.len(), 1);
        let trait_symbol = &symbols[0];
        assert_eq!(trait_symbol.name, "SomeTrait");
        assert_eq!(trait_symbol.kind, SymbolKind::INTERFACE);
        assert_eq!(
            test_utils::text_at(src, trait_symbol.range),
            "trait SomeTrait<U> {\n    fn some_method(x: U);\n}"
        );
        assert_eq!(test_utils::text_at(src, trait_symbol.selection_range), "SomeTrait");

        let children = trait_symbol.children.as_ref().expect("Expected children");
        assert_eq!(children.len(), 1);
        let method = &children[0];
        assert_eq!(method.name, "some_method");
        assert_eq!(method.kind, SymbolKind::METHOD);
        // For a trait method declaration, `range` starts at the method name (not `fn`).
        assert_eq!(test_utils::text_at(src, method.range), "some_method(x: U);");
        assert_eq!(test_utils::text_at(src, method.selection_range), "some_method");
    }

    #[test]
    fn test_document_symbol_for_trait_impl() {
        let src = r#"struct SomeStruct {}

trait SomeTrait<U> {
    fn some_method(x: U);
}

impl SomeTrait<i32> for SomeStruct {
    fn some_method(_x: i32) {
    }
}
"#;
        let symbols = get_document_symbols(src);

        // [struct SomeStruct, trait SomeTrait, impl SomeTrait<i32> for SomeStruct]
        assert_eq!(symbols.len(), 3);
        let impl_symbol = &symbols[2];
        assert_eq!(impl_symbol.name, "impl SomeTrait<i32> for SomeStruct");
        assert_eq!(impl_symbol.kind, SymbolKind::NAMESPACE);
        assert_eq!(
            test_utils::text_at(src, impl_symbol.range),
            "impl SomeTrait<i32> for SomeStruct {\n    fn some_method(_x: i32) {\n    }\n}"
        );
        // For a trait impl, `selection_range` points at the trait name (not the target type).
        assert_eq!(test_utils::text_at(src, impl_symbol.selection_range), "SomeTrait");

        let children = impl_symbol.children.as_ref().expect("Expected children");
        assert_eq!(children.len(), 1);
        let method = &children[0];
        assert_eq!(method.name, "some_method");
        assert_eq!(method.detail.as_deref(), Some("fn some_method(_x: i32)"));
        assert_eq!(method.kind, SymbolKind::FUNCTION);
    }

    #[test]
    fn test_document_symbol_for_module_with_global() {
        let src = r#"mod submodule {
    global SOME_GLOBAL = 1;
}
"#;
        let symbols = get_document_symbols(src);

        assert_eq!(symbols.len(), 1);
        let module = &symbols[0];
        assert_eq!(module.name, "submodule");
        assert_eq!(module.kind, SymbolKind::MODULE);
        assert_eq!(
            test_utils::text_at(src, module.range),
            "mod submodule {\n    global SOME_GLOBAL = 1;\n}"
        );
        assert_eq!(test_utils::text_at(src, module.selection_range), "submodule");

        let children = module.children.as_ref().expect("Expected children");
        assert_eq!(children.len(), 1);
        let global = &children[0];
        assert_eq!(global.name, "SOME_GLOBAL");
        assert_eq!(global.kind, SymbolKind::CONSTANT);
        assert_eq!(test_utils::text_at(src, global.range), "global SOME_GLOBAL = 1;");
        assert_eq!(test_utils::text_at(src, global.selection_range), "SOME_GLOBAL");
    }

    #[test]
    fn test_document_symbol_for_primitive_impl() {
        let src = "impl i32 {}\n";
        let symbols = get_document_symbols(src);

        assert_eq!(symbols.len(), 1);
        let symbol = &symbols[0];
        assert_eq!(symbol.name, "i32");
        assert_eq!(symbol.kind, SymbolKind::NAMESPACE);
        assert_eq!(test_utils::text_at(src, symbol.range), "impl i32 {}");
        assert_eq!(test_utils::text_at(src, symbol.selection_range), "i32");
        assert_eq!(symbol.children.as_deref(), Some(&[][..]));
    }

    #[test]
    fn test_document_symbol_for_type_alias() {
        let src = "type MyAlias = (i32, bool);\n";
        let symbols = get_document_symbols(src);

        assert_eq!(symbols.len(), 1);
        let symbol = &symbols[0];
        assert_eq!(symbol.name, "MyAlias");
        assert_eq!(symbol.kind, SymbolKind::TYPE_PARAMETER);
        assert!(symbol.children.is_none());
        assert_eq!(test_utils::text_at(src, symbol.range), "type MyAlias = (i32, bool);");
        assert_eq!(test_utils::text_at(src, symbol.selection_range), "MyAlias");
    }

    #[test]
    fn test_document_symbol_for_enum_with_variants() {
        let src = r#"enum Color {
    Red,
    Rgb(u8, u8, u8),
}
"#;
        let symbols = get_document_symbols(src);

        assert_eq!(symbols.len(), 1);
        let symbol = &symbols[0];
        assert_eq!(symbol.name, "Color");
        assert_eq!(symbol.kind, SymbolKind::ENUM);
        assert_eq!(
            test_utils::text_at(src, symbol.range),
            "enum Color {\n    Red,\n    Rgb(u8, u8, u8),\n}"
        );
        assert_eq!(test_utils::text_at(src, symbol.selection_range), "Color");

        let children = symbol.children.as_ref().expect("Expected children");
        assert_eq!(children.len(), 2);

        let variant = &children[0];
        assert_eq!(variant.name, "Red");
        assert_eq!(variant.kind, SymbolKind::ENUM_MEMBER);
        assert_eq!(test_utils::text_at(src, variant.range), "Red");
        assert_eq!(test_utils::text_at(src, variant.selection_range), "Red");

        let variant = &children[1];
        assert_eq!(variant.name, "Rgb");
        assert_eq!(variant.kind, SymbolKind::ENUM_MEMBER);
        // The variant's range extends through the end of its last parameter type.
        assert_eq!(test_utils::text_at(src, variant.range), "Rgb(u8, u8, u8");
        assert_eq!(test_utils::text_at(src, variant.selection_range), "Rgb");
    }

    #[test]
    fn test_document_symbol_for_module_declaration() {
        let src = "mod foo;\n";
        let symbols = get_document_symbols(src);

        assert_eq!(symbols.len(), 1);
        let symbol = &symbols[0];
        assert_eq!(symbol.name, "foo");
        assert_eq!(symbol.kind, SymbolKind::MODULE);
        assert!(symbol.children.is_none());
        assert_eq!(test_utils::text_at(src, symbol.range), "mod foo;");
        assert_eq!(test_utils::text_at(src, symbol.selection_range), "foo");
    }

    #[test]
    fn test_function_with_just_open_parentheses() {
        let src = "fn main(\n";
        let mut symbols = get_document_symbols(src);
        assert_eq!(symbols.len(), 1);
        let symbol = symbols.remove(0);
        // Parse-recovery: the symbol's range extends from `fn` to the end of the only line
        // (the function never gets a proper close), and its selection_range is the name.
        assert_eq!(test_utils::text_at(src, symbol.range), "fn main(\n");
        assert_eq!(test_utils::text_at(src, symbol.selection_range), "main");
    }
}
