use noirc_frontend::{
    ast::{NoirTrait, Param, Pattern, TraitItem, Visibility},
    token::{Keyword, Token},
};

use super::{chunks::Chunks, function::FunctionToFormat, Formatter};

impl<'a> Formatter<'a> {
    pub(super) fn format_trait(&mut self, noir_trait: NoirTrait) {
        if !noir_trait.attributes.is_empty() {
            self.format_attributes();
        }
        self.write_indentation();
        self.format_item_visibility(noir_trait.visibility);
        self.write_keyword(Keyword::Trait);
        self.write_space();
        self.write_identifier(noir_trait.name);
        self.format_generics(noir_trait.generics);

        if !noir_trait.bounds.is_empty() {
            self.skip_comments_and_whitespace();
            self.write_token(Token::Colon);
            self.write_space();

            for (index, trait_bound) in noir_trait.bounds.into_iter().enumerate() {
                if index > 0 {
                    self.write_space();
                    self.write_token(Token::Plus);
                    self.write_space();
                }
                self.format_trait_bound(trait_bound);
            }
        }

        self.write_space();
        self.write_left_brace();
        if noir_trait.items.is_empty() {
            self.increase_indentation();
            self.skip_comments_and_whitespace();
            self.decrease_indentation();
        } else {
            self.increase_indentation();
            self.write_line();

            for (index, documented_item) in noir_trait.items.into_iter().enumerate() {
                if index > 0 {
                    self.write_line();
                }

                let doc_comments = documented_item.doc_comments;
                let item = documented_item.item;
                if !doc_comments.is_empty() {
                    self.format_outer_doc_comments();
                }
                self.format_trait_item(item);
            }

            self.decrease_indentation();
            self.write_line();
            self.write_indentation();
        }
        self.write_right_brace();
    }

    fn format_trait_item(&mut self, item: TraitItem) {
        match item {
            TraitItem::Function {
                is_unconstrained: _,
                visibility,
                is_comptime: _,
                name,
                generics,
                parameters,
                return_type,
                where_clause,
                body,
            } => {
                let parameters = parameters
                    .into_iter()
                    .map(|(name, typ)| Param {
                        visibility: Visibility::Private,
                        pattern: Pattern::Identifier(name),
                        typ,
                        span: Default::default(), // Doesn't matter
                    })
                    .collect();

                let func = FunctionToFormat {
                    visibility,
                    name,
                    generics,
                    parameters,
                    return_type,
                    return_visibility: Visibility::Private,
                    where_clause,
                    body,
                };
                self.format_function_impl(func);
            }
            TraitItem::Constant { name, typ, default_value } => {
                let mut chunks = Chunks::new();
                chunks.text(self.chunk(|formatter| {
                    formatter.write_keyword(Keyword::Let);
                    formatter.write_space();
                    formatter.write_identifier(name);
                    formatter.write_token(Token::Colon);
                    formatter.write_space();
                    formatter.format_type(typ);
                }));

                if let Some(default_value) = default_value {
                    chunks.text(self.chunk(|formatter| {
                        formatter.write_space();
                        formatter.write_token(Token::Assign);
                    }));
                    chunks.increase_indentation();
                    chunks.space_or_line();
                    self.format_expression(default_value, &mut chunks);
                }

                chunks.text(self.chunk(|formatter| {
                    formatter.write_semicolon();
                }));

                self.write_indentation();
                self.format_chunks(chunks);
            }
            TraitItem::Type { name } => {
                self.write_indentation();
                self.write_keyword(Keyword::Type);
                self.write_space();
                self.write_identifier(name);
                self.write_semicolon();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_format;

    #[test]
    fn format_empty_trait() {
        let src = " mod moo { /// Comment
        pub  trait Foo {  } }";
        let expected = "mod moo {
    /// Comment
    pub trait Foo {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_empty_trait_with_generics() {
        let src = " mod moo { trait Foo < A, B, > {  } }";
        let expected = "mod moo {
    trait Foo<A, B> {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_trait_with_parents() {
        let src = " mod moo { trait Foo : Bar  +  Baz {  } }";
        let expected = "mod moo {
    trait Foo: Bar + Baz {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_trait_with_type() {
        let src = " mod moo { trait Foo { 
    /// hello
            type X;
         } }";
        let expected = "mod moo {
    trait Foo {
        /// hello
        type X;
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_trait_with_constant_no_value() {
        let src = " mod moo { trait Foo { 
            let  x  : i32 ;
         } }";
        let expected = "mod moo {
    trait Foo {
        let x: i32;
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_trait_with_constant_with_value() {
        let src = " mod moo { trait Foo { 
            let  x  : i32  =  1 ;
         } }";
        let expected = "mod moo {
    trait Foo {
        let x: i32 = 1;
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_trait_with_function_without_body() {
        let src = " mod moo { trait Foo { 
    /// hello 
            pub  fn  foo ( );
         } }";
        let expected = "mod moo {
    trait Foo {
        /// hello
        pub fn foo();
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_trait_with_function_with_body() {
        let src = " mod moo { trait Foo { 
    /// hello 
            pub  fn  foo ( ) { 1 }
         } }";
        let expected = "mod moo {
    trait Foo {
        /// hello
        pub fn foo() {
            1
        }
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_trait_with_function_with_params() {
        let src = " mod moo { trait Foo { 
    /// hello 
            pub  fn  foo ( x : i32 , y : Field );
         } }";
        let expected = "mod moo {
    trait Foo {
        /// hello
        pub fn foo(x: i32, y: Field);
    }
}
";
        assert_format(src, expected);
    }
}
