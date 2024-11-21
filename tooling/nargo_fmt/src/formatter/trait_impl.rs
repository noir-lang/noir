use noirc_frontend::{
    ast::{
        FunctionDefinition, ItemVisibility, NoirFunction, NoirTraitImpl, Pattern, TraitImplItem,
        TraitImplItemKind,
    },
    token::{Keyword, Token},
};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_trait_impl(&mut self, trait_impl: NoirTraitImpl) {
        // skip synthetic trait impl's, e.g. generated from trait aliases
        if trait_impl.is_synthetic {
            return;
        }

        let has_where_clause = !trait_impl.where_clause.is_empty();

        self.write_indentation();
        self.write_keyword(Keyword::Impl);
        self.format_generics(trait_impl.impl_generics);
        self.write_space();
        self.format_path(trait_impl.trait_name);
        self.format_generic_type_args(trait_impl.trait_generics);
        self.write_space();
        self.write_keyword(Keyword::For);
        self.write_space();
        self.format_type(trait_impl.object_type);

        if has_where_clause {
            self.format_where_clause(
                trait_impl.where_clause,
                true, // write trailing comma and newline
            );
        } else {
            self.write_space();
        }
        self.write_left_brace();

        if trait_impl.items.is_empty() {
            self.format_empty_block_contents();
        } else {
            self.increase_indentation();
            self.write_line();

            for (index, documented_item) in trait_impl.items.into_iter().enumerate() {
                if index > 0 {
                    self.write_line();
                }

                let doc_comments = documented_item.doc_comments;
                let item = documented_item.item;
                if !doc_comments.is_empty() {
                    self.format_outer_doc_comments();
                }
                self.format_trait_impl_item(item);
            }

            self.skip_comments_and_whitespace();
            self.decrease_indentation();
            self.write_line();
            self.write_indentation();
        }

        self.write_right_brace();
    }

    fn format_trait_impl_item(&mut self, item: TraitImplItem) {
        match item.kind {
            TraitImplItemKind::Function(noir_function) => {
                // Trait impl functions are public, but there's no `pub` keyword in the source code,
                // so to format it we pass a private one.
                let def =
                    FunctionDefinition { visibility: ItemVisibility::Private, ..noir_function.def };
                let noir_function = NoirFunction { def, ..noir_function };
                self.format_function(noir_function);
            }
            TraitImplItemKind::Constant(name, typ, value) => {
                let pattern = Pattern::Identifier(name);
                let chunks = self.chunk_formatter().format_let_or_global(
                    Keyword::Let,
                    pattern,
                    typ,
                    Some(value),
                    Vec::new(), // Attributes
                );

                self.write_indentation();
                self.format_chunk_group(chunks);
            }
            TraitImplItemKind::Type { name, alias } => {
                self.write_indentation();
                self.write_keyword(Keyword::Type);
                self.write_space();
                self.write_identifier(name);
                self.write_space();
                self.write_token(Token::Assign);
                self.write_space();
                self.format_type(alias);
                self.write_semicolon();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_format;

    #[test]
    fn format_empty_trait_impl() {
        let src = " mod moo { impl  Foo  for  Bar {  } }";
        let expected = "mod moo {
    impl Foo for Bar {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_empty_trait_impl_with_trait_generics() {
        let src = " mod moo { impl  Foo < T >   for  Bar {  } }";
        let expected = "mod moo {
    impl Foo<T> for Bar {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_empty_trait_impl_with_impl_generics() {
        let src = " mod moo { impl<T> Default for Option {
} }";
        let expected = "mod moo {
    impl<T> Default for Option {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_empty_trait_impl_with_where_clause() {
        let src = " mod moo { impl  Foo < T >   for  Bar  where  T : Baz {  } }";
        let expected = "mod moo {
    impl Foo<T> for Bar
    where
        T: Baz,
    {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_empty_trait_impl_with_where_clause_with_trait_bound_generics() {
        let src = "impl<T, U> Into<T> for U where T: From<U> { }";
        let expected = "impl<T, U> Into<T> for U
where
    T: From<U>,
{}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_trait_impl_function() {
        let src = " mod moo { impl  Foo  for  Bar {  
        /// Some doc comment
fn foo ( ) { }
         } }";
        let expected = "mod moo {
    impl Foo for Bar {
        /// Some doc comment
        fn foo() {}
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_trait_impl_constant_without_type() {
        let src = " mod moo { impl  Foo  for  Bar {  
            let X =42 ;
         } }";
        let expected = "mod moo {
    impl Foo for Bar {
        let X = 42;
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_trait_impl_constant_with_type() {
        let src = " mod moo { impl  Foo  for  Bar {  
            let X : i32=42 ;
         } }";
        let expected = "mod moo {
    impl Foo for Bar {
        let X: i32 = 42;
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_trait_impl_type() {
        let src = " mod moo { impl  Foo  for  Bar {  
            type  X  =  i32 ;
         } }";
        let expected = "mod moo {
    impl Foo for Bar {
        type X = i32;
    }
}
";
        assert_format(src, expected);
    }
}
