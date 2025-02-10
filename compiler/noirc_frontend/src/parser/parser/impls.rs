use noirc_errors::Span;

use crate::{
    ast::{
        Documented, Expression, ExpressionKind, GenericTypeArgs, Ident, ItemVisibility,
        NoirFunction, NoirTraitImpl, Path, TraitImplItem, TraitImplItemKind, TypeImpl,
        UnresolvedGeneric, UnresolvedType, UnresolvedTypeData,
    },
    parser::{labels::ParsingRuleLabel, ParserErrorReason},
    token::{Keyword, Token},
};

use super::{parse_many::without_separator, Parser};

pub(crate) enum Impl {
    Impl(TypeImpl),
    TraitImpl(NoirTraitImpl),
}

impl<'a> Parser<'a> {
    /// Impl
    ///     = TypeImpl
    ///     | TraitImpl
    pub(crate) fn parse_impl(&mut self) -> Impl {
        let generics = self.parse_generics();

        let type_span_start = self.current_token_span;
        let object_type = self.parse_type_or_error();
        let type_span = self.span_since(type_span_start);

        if self.eat_keyword(Keyword::For) {
            if let UnresolvedTypeData::Named(trait_name, trait_generics, _) = object_type.typ {
                return Impl::TraitImpl(self.parse_trait_impl(
                    generics,
                    trait_generics,
                    trait_name,
                ));
            } else {
                self.push_error(
                    ParserErrorReason::ExpectedTrait { found: object_type.typ.to_string() },
                    self.current_token_span,
                );

                // Error, but we continue parsing the type and assume this is going to be a regular type impl
                self.parse_type();
            };
        }

        self.parse_type_impl(object_type, type_span, generics)
    }

    /// TypeImpl = 'impl' Generics Type TypeImplBody
    fn parse_type_impl(
        &mut self,
        object_type: UnresolvedType,
        type_span: Span,
        generics: Vec<UnresolvedGeneric>,
    ) -> Impl {
        let where_clause = self.parse_where_clause();
        let methods = self.parse_type_impl_body();

        Impl::Impl(TypeImpl { object_type, type_span, generics, where_clause, methods })
    }

    /// TypeImplBody = '{' TypeImplItem* '}'
    ///
    /// TypeImplItem = OuterDocComments Attributes Modifiers Function
    fn parse_type_impl_body(&mut self) -> Vec<(Documented<NoirFunction>, Span)> {
        if !self.eat_left_brace() {
            self.expected_token(Token::LeftBrace);
            return Vec::new();
        }

        self.parse_many(
            "type impl methods",
            without_separator().until(Token::RightBrace),
            Self::parse_type_impl_method,
        )
    }

    fn parse_type_impl_method(&mut self) -> Option<(Documented<NoirFunction>, Span)> {
        self.parse_item_in_list(ParsingRuleLabel::Function, |parser| {
            let doc_comments = parser.parse_outer_doc_comments();
            let start_span = parser.current_token_span;
            let attributes = parser.parse_attributes();
            let modifiers = parser.parse_modifiers(
                false, // allow mutable
            );

            if parser.eat_keyword(Keyword::Fn) {
                let method = parser.parse_function(
                    attributes,
                    modifiers.visibility,
                    modifiers.comptime.is_some(),
                    modifiers.unconstrained.is_some(),
                    true, // allow_self
                );
                Some((Documented::new(method, doc_comments), parser.span_since(start_span)))
            } else {
                parser.modifiers_not_followed_by_an_item(modifiers);
                None
            }
        })
    }

    /// TraitImpl = 'impl' Generics Path GenericTypeArgs 'for' Type TraitImplBody
    fn parse_trait_impl(
        &mut self,
        impl_generics: Vec<UnresolvedGeneric>,
        trait_generics: GenericTypeArgs,
        trait_name: Path,
    ) -> NoirTraitImpl {
        let object_type = self.parse_type_or_error();
        let where_clause = self.parse_where_clause();
        let items = self.parse_trait_impl_body();
        let is_synthetic = false;

        NoirTraitImpl {
            impl_generics,
            trait_name,
            trait_generics,
            object_type,
            where_clause,
            items,
            is_synthetic,
        }
    }

    /// TraitImplBody = '{' TraitImplItem* '}'
    fn parse_trait_impl_body(&mut self) -> Vec<Documented<TraitImplItem>> {
        if !self.eat_left_brace() {
            self.expected_token(Token::LeftBrace);
            return Vec::new();
        }

        self.parse_many(
            "trait impl item",
            without_separator().until(Token::RightBrace),
            Self::parse_trait_impl_item,
        )
    }

    fn parse_trait_impl_item(&mut self) -> Option<Documented<TraitImplItem>> {
        self.parse_item_in_list(ParsingRuleLabel::TraitImplItem, |parser| {
            let start_span = parser.current_token_span;
            let doc_comments = parser.parse_outer_doc_comments();

            if let Some(kind) = parser.parse_trait_impl_item_kind() {
                let item = TraitImplItem { kind, span: parser.span_since(start_span) };
                Some(Documented::new(item, doc_comments))
            } else {
                None
            }
        })
    }

    /// TraitImplItem
    ///     = TraitImplType
    ///     | TraitImplConstant
    ///     | TraitImplFunction
    fn parse_trait_impl_item_kind(&mut self) -> Option<TraitImplItemKind> {
        if let Some(kind) = self.parse_trait_impl_type() {
            return Some(kind);
        }

        if let Some(kind) = self.parse_trait_impl_constant() {
            return Some(kind);
        }

        self.parse_trait_impl_function()
    }

    /// TraitImplType = 'type' identifier ( ':' Type )? ';'
    fn parse_trait_impl_type(&mut self) -> Option<TraitImplItemKind> {
        if !self.eat_keyword(Keyword::Type) {
            return None;
        }

        let Some(name) = self.eat_ident() else {
            self.expected_identifier();
            self.eat_semicolons();
            return Some(TraitImplItemKind::Type {
                name: Ident::default(),
                alias: UnresolvedType { typ: UnresolvedTypeData::Error, span: Span::default() },
            });
        };

        let alias = if self.eat_assign() {
            self.parse_type_or_error()
        } else {
            UnresolvedType { typ: UnresolvedTypeData::Error, span: Span::default() }
        };

        self.eat_semicolons();

        Some(TraitImplItemKind::Type { name, alias })
    }

    /// TraitImplConstant = 'let' identifier OptionalTypeAnnotation ';'
    fn parse_trait_impl_constant(&mut self) -> Option<TraitImplItemKind> {
        if !self.eat_keyword(Keyword::Let) {
            return None;
        }

        let name = match self.eat_ident() {
            Some(name) => name,
            None => {
                self.expected_identifier();
                Ident::default()
            }
        };

        let typ = self.parse_optional_type_annotation();

        let expr = if self.eat_assign() {
            self.parse_expression_or_error()
        } else {
            self.expected_token(Token::Assign);
            Expression { kind: ExpressionKind::Error, span: Span::default() }
        };

        self.eat_semicolons();

        Some(TraitImplItemKind::Constant(name, typ, expr))
    }

    /// TraitImplFunction = Attributes Modifiers Function
    fn parse_trait_impl_function(&mut self) -> Option<TraitImplItemKind> {
        let attributes = self.parse_attributes();

        let modifiers = self.parse_modifiers(
            false, // allow mut
        );
        if modifiers.visibility != ItemVisibility::Private {
            self.push_error(
                ParserErrorReason::TraitImplVisibilityIgnored,
                modifiers.visibility_span,
            );
        }

        if !self.eat_keyword(Keyword::Fn) {
            self.modifiers_not_followed_by_an_item(modifiers);
            return None;
        }

        let noir_function = self.parse_function(
            attributes,
            modifiers.visibility,
            modifiers.comptime.is_some(),
            modifiers.unconstrained.is_some(),
            true, // allow_self
        );
        Some(TraitImplItemKind::Function(noir_function))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            ItemVisibility, NoirTraitImpl, Pattern, TraitImplItemKind, TypeImpl, UnresolvedTypeData,
        },
        parser::{
            parser::{
                parse_program,
                tests::{expect_no_errors, get_single_error, get_source_with_error_span},
            },
            ItemKind,
        },
    };

    fn parse_type_impl_no_errors(src: &str) -> TypeImpl {
        let (mut module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::Impl(type_impl) = item.kind else {
            panic!("Expected type impl");
        };
        type_impl
    }

    fn parse_trait_impl_no_errors(src: &str) -> NoirTraitImpl {
        let (mut module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = module.items.remove(0);
        let ItemKind::TraitImpl(noir_trait_impl) = item.kind else {
            panic!("Expected trait impl");
        };
        noir_trait_impl
    }

    #[test]
    fn parse_empty_impl() {
        let src = "impl Foo {}";
        let type_impl = parse_type_impl_no_errors(src);
        assert_eq!(type_impl.object_type.to_string(), "Foo");
        assert!(type_impl.generics.is_empty());
        assert!(type_impl.methods.is_empty());
    }

    #[test]
    fn parse_empty_impl_with_generics() {
        let src = "impl <A, B> Foo {}";
        let type_impl = parse_type_impl_no_errors(src);
        assert_eq!(type_impl.object_type.to_string(), "Foo");
        assert_eq!(type_impl.generics.len(), 2);
        assert!(type_impl.methods.is_empty());
    }

    #[test]
    fn parse_impl_with_methods() {
        let src = "impl Foo { unconstrained fn foo() {} pub comptime fn bar() {} }";
        let mut type_impl = parse_type_impl_no_errors(src);
        assert_eq!(type_impl.object_type.to_string(), "Foo");
        assert_eq!(type_impl.methods.len(), 2);

        let (method, _) = type_impl.methods.remove(0);
        let method = method.item;
        assert_eq!(method.def.name.to_string(), "foo");
        assert!(method.def.is_unconstrained);
        assert!(!method.def.is_comptime);
        assert_eq!(method.def.visibility, ItemVisibility::Private);

        let (method, _) = type_impl.methods.remove(0);
        let method = method.item;
        assert_eq!(method.def.name.to_string(), "bar");
        assert!(method.def.is_comptime);
        assert_eq!(method.def.visibility, ItemVisibility::Public);
    }

    #[test]
    fn parse_impl_with_attribute_on_method() {
        let src = "
        impl Foo {
            #[something]
            fn foo(self) {}
        }
        ";
        let type_impl = parse_type_impl_no_errors(src);
        let attributes = type_impl.methods[0].0.item.attributes();
        assert_eq!(attributes.secondary.len(), 1);
    }

    #[test]
    fn parse_impl_with_self_argument() {
        let src = "impl Foo { fn foo(self) {} }";
        let mut type_impl = parse_type_impl_no_errors(src);
        assert_eq!(type_impl.methods.len(), 1);

        let (method, _) = type_impl.methods.remove(0);
        let mut method = method.item;
        assert_eq!(method.def.name.to_string(), "foo");
        assert_eq!(method.def.parameters.len(), 1);

        let param = method.def.parameters.remove(0);
        let Pattern::Identifier(name) = param.pattern else {
            panic!("Expected identifier pattern");
        };
        assert_eq!(name.to_string(), "self");
        assert_eq!(param.typ.to_string(), "Self");
    }

    #[test]
    fn parse_impl_with_mut_self_argument() {
        let src = "impl Foo { fn foo(mut self) {} }";
        let mut type_impl = parse_type_impl_no_errors(src);
        assert_eq!(type_impl.methods.len(), 1);

        let (method, _) = type_impl.methods.remove(0);
        let mut method = method.item;
        assert_eq!(method.def.name.to_string(), "foo");
        assert_eq!(method.def.parameters.len(), 1);

        let param = method.def.parameters.remove(0);
        let Pattern::Mutable(pattern, _, true) = param.pattern else {
            panic!("Expected mutable pattern");
        };
        let pattern: &Pattern = &pattern;
        let Pattern::Identifier(name) = pattern else {
            panic!("Expected identifier pattern");
        };
        assert_eq!(name.to_string(), "self");
        assert_eq!(param.typ.to_string(), "Self");
    }

    #[test]
    fn parse_impl_with_reference_mut_self_argument() {
        let src = "impl Foo { fn foo(&mut self) {} }";
        let mut type_impl = parse_type_impl_no_errors(src);
        assert_eq!(type_impl.methods.len(), 1);

        let (method, _) = type_impl.methods.remove(0);
        let mut method = method.item;
        assert_eq!(method.def.name.to_string(), "foo");
        assert_eq!(method.def.parameters.len(), 1);

        let param = method.def.parameters.remove(0);
        let Pattern::Identifier(name) = param.pattern else {
            panic!("Expected identifier pattern");
        };
        assert_eq!(name.to_string(), "self");
        assert_eq!(param.typ.to_string(), "&mut Self");
    }

    #[test]
    fn parse_impl_with_self_argument_followed_by_type() {
        let src = "impl Foo { fn foo(self: Foo) {} }";
        let mut type_impl = parse_type_impl_no_errors(src);
        assert_eq!(type_impl.methods.len(), 1);

        let (method, _) = type_impl.methods.remove(0);
        let mut method = method.item;
        assert_eq!(method.def.name.to_string(), "foo");
        assert_eq!(method.def.parameters.len(), 1);

        let param = method.def.parameters.remove(0);
        let Pattern::Identifier(name) = param.pattern else {
            panic!("Expected identifier pattern");
        };
        assert_eq!(name.to_string(), "self");
        assert_eq!(param.typ.to_string(), "Foo");
    }

    #[test]
    fn parse_empty_impl_missing_right_brace() {
        let src = "impl Foo {";
        let (module, errors) = parse_program(src);
        assert_eq!(errors.len(), 1);
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Impl(type_impl) = &item.kind else {
            panic!("Expected type impl");
        };
        assert_eq!(type_impl.object_type.to_string(), "Foo");
    }

    #[test]
    fn parse_empty_impl_incorrect_body() {
        let src = "impl Foo { hello fn foo() {} }";
        let (module, errors) = parse_program(src);
        assert_eq!(errors.len(), 1);
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Impl(type_impl) = &item.kind else {
            panic!("Expected type impl");
        };
        assert_eq!(type_impl.object_type.to_string(), "Foo");
        assert_eq!(type_impl.methods.len(), 1);
    }

    #[test]
    fn parse_empty_trait_impl() {
        let src = "impl Foo for Field {}";
        let trait_impl = parse_trait_impl_no_errors(src);
        assert_eq!(trait_impl.trait_name.to_string(), "Foo");
        assert!(matches!(trait_impl.object_type.typ, UnresolvedTypeData::FieldElement));
        assert!(trait_impl.items.is_empty());
        assert!(trait_impl.impl_generics.is_empty());
    }

    #[test]
    fn parse_empty_trait_impl_with_generics() {
        let src = "impl <T> Foo for Field {}";
        let trait_impl = parse_trait_impl_no_errors(src);
        assert_eq!(trait_impl.trait_name.to_string(), "Foo");
        assert!(matches!(trait_impl.object_type.typ, UnresolvedTypeData::FieldElement));
        assert!(trait_impl.items.is_empty());
        assert_eq!(trait_impl.impl_generics.len(), 1);
    }

    #[test]
    fn parse_trait_impl_with_function() {
        let src = "impl Foo for Field { fn foo() {} }";
        let mut trait_impl = parse_trait_impl_no_errors(src);
        assert_eq!(trait_impl.trait_name.to_string(), "Foo");
        assert_eq!(trait_impl.items.len(), 1);

        let item = trait_impl.items.remove(0).item;
        let TraitImplItemKind::Function(function) = item.kind else {
            panic!("Expected function");
        };
        assert_eq!(function.def.name.to_string(), "foo");
        assert_eq!(function.def.visibility, ItemVisibility::Private);
    }

    #[test]
    fn parse_trait_impl_with_generic_type_args() {
        let src = "impl Foo<i32, X = Field> for Field { }";
        let trait_impl = parse_trait_impl_no_errors(src);
        assert_eq!(trait_impl.trait_name.to_string(), "Foo");
        assert!(!trait_impl.trait_generics.is_empty());
    }

    #[test]
    fn parse_trait_impl_with_type() {
        let src = "impl Foo for Field { type Foo = i32; }";
        let mut trait_impl = parse_trait_impl_no_errors(src);
        assert_eq!(trait_impl.trait_name.to_string(), "Foo");
        assert_eq!(trait_impl.items.len(), 1);

        let item = trait_impl.items.remove(0).item;
        let TraitImplItemKind::Type { name, alias } = item.kind else {
            panic!("Expected type");
        };
        assert_eq!(name.to_string(), "Foo");
        assert_eq!(alias.to_string(), "i32");
    }

    #[test]
    fn parse_trait_impl_with_let() {
        let src = "impl Foo for Field { let x: Field = 1; }";
        let mut trait_impl = parse_trait_impl_no_errors(src);
        assert_eq!(trait_impl.trait_name.to_string(), "Foo");
        assert_eq!(trait_impl.items.len(), 1);

        let item = trait_impl.items.remove(0).item;
        let TraitImplItemKind::Constant(name, typ, expr) = item.kind else {
            panic!("Expected constant");
        };
        assert_eq!(name.to_string(), "x");
        assert_eq!(typ.to_string(), "Field");
        assert_eq!(expr.to_string(), "1");
    }

    #[test]
    fn recovers_on_unknown_impl_item() {
        let src = "
        impl Foo { hello fn foo() {} }
                   ^^^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (module, errors) = parse_program(&src);

        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Impl(type_impl) = &item.kind else {
            panic!("Expected impl");
        };
        assert_eq!(type_impl.methods.len(), 1);

        let error = get_single_error(&errors, span);
        assert_eq!(error.to_string(), "Expected a function but found 'hello'");
    }

    #[test]
    fn recovers_on_unknown_trait_impl_item() {
        let src = "
        impl Foo for i32 { hello fn foo() {} }
                           ^^^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let (module, errors) = parse_program(&src);

        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::TraitImpl(trait_imp) = &item.kind else {
            panic!("Expected trait impl");
        };
        assert_eq!(trait_imp.items.len(), 1);

        let error = get_single_error(&errors, span);
        assert_eq!(error.to_string(), "Expected a trait impl item but found 'hello'");
    }
}
