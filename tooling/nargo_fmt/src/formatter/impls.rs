use noirc_frontend::{ast::TypeImpl, token::Keyword};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_impl(&mut self, type_impl: TypeImpl) {
        let has_where_clause = !type_impl.where_clause.is_empty();

        self.write_indentation();
        self.write_keyword(Keyword::Impl);
        self.format_generics(type_impl.generics);
        self.write_space();
        self.format_type(type_impl.object_type);

        if has_where_clause {
            self.format_where_clause(
                type_impl.where_clause,
                true, // write trailing comma and newline
            );
        } else {
            self.write_space();
        }
        self.write_left_brace();

        if type_impl.methods.is_empty() {
            self.format_empty_block_contents();
        } else {
            self.increase_indentation();
            self.write_line();

            for (index, (documented_method, _span)) in type_impl.methods.into_iter().enumerate() {
                if index > 0 {
                    self.write_line();
                }

                let doc_comments = documented_method.doc_comments;
                let method = documented_method.item;
                if !doc_comments.is_empty() {
                    self.format_outer_doc_comments();
                }
                self.format_function(
                    method, false, // skip visibility
                );
            }

            self.skip_comments_and_whitespace();
            self.decrease_indentation();
            self.write_line();
            self.write_indentation();
        }

        self.write_right_brace();
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_format;

    #[test]
    fn format_empty_impl() {
        let src = " mod moo { impl Foo {  } }";
        let expected = "mod moo {
    impl Foo {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_empty_impl_with_generics() {
        let src = " mod moo { impl < A,  B > Foo < A, B > {  } }";
        let expected = "mod moo {
    impl<A, B> Foo<A, B> {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_empty_impl_with_where_clause() {
        let src = " mod moo { impl Foo where A: B  {  } }";
        let expected = "mod moo {
    impl Foo
    where
        A: B,
    {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_impl_with_functions() {
        let src = " mod moo { impl Foo {  
/// hello
pub fn foo () { 1 }

/// world
pub ( crate ) fn bar () {
    }

fn one(self) {}
fn two(mut self) {}
fn three(&mut self) {}
 } }";
        let expected = "mod moo {
    impl Foo {
        /// hello
        pub fn foo() {
            1
        }

        /// world
        pub(crate) fn bar() {}

        fn one(self) {}
        fn two(mut self) {}
        fn three(&mut self) {}
    }
}
";
        assert_format(src, expected);
    }
}
