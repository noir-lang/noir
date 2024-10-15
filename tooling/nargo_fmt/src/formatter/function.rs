use noirc_frontend::{
    ast::{
        BlockExpression, FunctionReturnType, ItemVisibility, NoirFunction, Param, TraitBound,
        UnresolvedTraitConstraint, Visibility,
    },
    token::{Keyword, Token},
};

use super::{
    chunks::{Chunks, TextChunk},
    Formatter,
};

impl<'a> Formatter<'a> {
    pub(super) fn format_function(&mut self, func: NoirFunction) {
        self.format_attributes();
        self.write_indentation();
        self.format_function_modifiers(func.def.visibility);
        self.write_keyword(Keyword::Fn);
        self.write_space();
        self.write_identifier(func.def.name);
        self.format_generics(func.def.generics);
        self.write_left_paren();

        // When the function has no parameters we can format everything in a single line
        if func.def.parameters.is_empty() {
            self.increase_indentation();
            self.skip_comments_and_whitespace();
            self.decrease_indentation();
            self.format_function_right_paren_until_left_brace(
                func.def.return_type,
                func.def.return_visibility,
                &func.def.where_clause,
            );
        } else {
            let mut chunks = Chunks::new();
            chunks.increase_indentation();
            chunks.line();

            self.format_function_parameters(func.def.parameters, &mut chunks);

            chunks.decrease_indentation();
            chunks.line();

            chunks.text(self.chunk(|formatter| {
                formatter.format_function_right_paren_until_left_brace(
                    func.def.return_type,
                    func.def.return_visibility,
                    &func.def.where_clause,
                );
            }));

            self.format_chunks(chunks);
        }

        self.format_function_where_clause(func.def.where_clause);
        self.format_function_body(func.def.body);
        self.write_right_brace();
        self.write_line();
    }

    fn format_function_modifiers(&mut self, visibility: ItemVisibility) {
        // For backwards compatibility, unconstrained might come before visibility.
        // We'll remember this but put it after the visibility.
        let unconstrained = if self.token == Token::Keyword(Keyword::Unconstrained) {
            self.bump();
            self.skip_comments_and_whitespace();
            true
        } else {
            false
        };

        self.format_item_visibility(visibility);

        if unconstrained {
            self.write("unconstrained ");
        } else if self.token == Token::Keyword(Keyword::Unconstrained) {
            self.write_keyword(Keyword::Unconstrained);
            self.write_space();
        }

        if self.token == Token::Keyword(Keyword::Comptime) {
            self.write_keyword(Keyword::Comptime);
            self.write_space();
        }
    }

    fn format_function_parameters(&mut self, parameters: Vec<Param>, chunks: &mut Chunks) {
        for (index, param) in parameters.into_iter().enumerate() {
            if index > 0 {
                chunks.text(self.chunk(|formatter| {
                    formatter.write_comma();
                }));
                chunks.trailing_comment(self.skip_comments_and_whitespace_chunk());
                chunks.space_or_line();
            }

            chunks.text(self.chunk(|formatter| {
                formatter.format_pattern(param.pattern);
                formatter.write_token(Token::Colon);
                formatter.write_space();
                formatter.format_visibility(param.visibility);
                formatter.format_type(param.typ);
            }));
        }

        let chunk = self.chunk(|formatter| {
            formatter.skip_comments_and_whitespace();

            // A trailing comma might happen
            if formatter.token == Token::Comma {
                formatter.bump();
                formatter.skip_comments_and_whitespace();
            }
        });

        // Make sure to put a trailing comma before the last parameter comments, if there were any
        chunks.text_if_multiline(TextChunk::new(",".to_string()));

        chunks.text(chunk);
    }

    fn format_function_right_paren_until_left_brace(
        &mut self,
        return_type: FunctionReturnType,
        visibility: Visibility,
        where_clause: &[UnresolvedTraitConstraint],
    ) {
        self.write_right_paren();
        self.format_function_return_type(return_type, visibility);
        self.skip_comments_and_whitespace();

        // If there's no where clause the left brace goes on the same line as the function signature
        if where_clause.is_empty() {
            // There might still be a where keyword that we'll remove
            if self.token == Token::Keyword(Keyword::Where) {
                self.bump();
                self.skip_comments_and_whitespace();
            }

            self.write_space();
            self.write_left_brace();
        }
    }

    fn format_function_return_type(
        &mut self,
        return_type: FunctionReturnType,
        visibility: Visibility,
    ) {
        match return_type {
            FunctionReturnType::Default(..) => (),
            FunctionReturnType::Ty(typ) => {
                self.write_space();
                self.write_token(Token::Arrow);
                self.write_space();
                self.format_visibility(visibility);
                self.format_type(typ);
            }
        }
    }

    fn format_function_where_clause(&mut self, constraints: Vec<UnresolvedTraitConstraint>) {
        if constraints.is_empty() {
            // TODO: there might still be a `where` token
            return;
        }

        self.skip_comments_and_whitespace();
        self.write_line();
        self.write_indentation();
        self.write_keyword(Keyword::Where);
        self.increase_indentation();

        // If we have `where F: Foo + Bar`, that's actually parsed as two constraints: `F: Foo` and `F: Bar`.
        // To format it we'll have to skip the second type `F` if we find a `+` token.
        let mut write_type = true;

        for constraint in constraints {
            if write_type {
                self.write_line();
                self.write_indentation();
                self.format_type(constraint.typ);
                self.write_token(Token::Colon);
                self.write_space();
            }

            self.format_trait_bound(constraint.trait_bound);
            self.skip_comments_and_whitespace();

            if self.token == Token::Plus {
                self.write_space();
                self.write_token(Token::Plus);
                self.write_space();
                write_type = false;
                continue;
            }

            write_type = true;

            if self.token == Token::Comma {
                self.write_token(Token::Comma);
            } else {
                self.write(",")
            }
        }

        self.decrease_indentation();
        self.write_line();
        self.write_indentation();
        self.write_left_brace();
    }

    fn format_function_body(&mut self, body: BlockExpression) {
        if body.is_empty() {
            self.increase_indentation();
            let skip_result = self.skip_comments_and_whitespace_writing_lines_if_found();
            self.decrease_indentation();
            if skip_result.wrote_comment {
                self.write_line();
                self.write_indentation();
            }
        } else {
            let mut chunks = Chunks::new();
            chunks.increase_indentation();
            chunks.line();

            let statements_len = body.statements.len();

            for (index, statement) in body.statements.into_iter().enumerate() {
                self.format_statement(statement, &mut chunks);
                if index == statements_len - 1 {
                    chunks.decrease_indentation();
                    chunks.line();
                } else {
                    chunks.lines(self.two_newlines_or_more_follow());
                }
            }

            self.format_chunks_in_multiple_lines(chunks);
        }
    }

    fn format_trait_bound(&mut self, trait_bound: TraitBound) {
        self.format_path(trait_bound.trait_path);
        // TODO: generics
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_format, assert_format_with_max_width};

    #[test]
    fn format_simple_function() {
        let src = "mod moo { 
        /// hello 
#[attr]  pub  fn  foo (  )  { }  }";
        let expected = "mod moo {
    /// hello
    #[attr]
    pub fn foo() {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_args() {
        let src = "fn  foo ( x:  i32 , y:i32 , )  { }  ";
        let expected = "fn foo(x: i32, y: i32) {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_args_that_exceed_max_width() {
        let src = "fn  foo ( this_is_long:  i32 , like_really_long:i32 , )  { }  ";
        let expected = "fn foo(
    this_is_long: i32,
    like_really_long: i32,
) {}\n";
        assert_format_with_max_width(src, expected, 40);
    }

    #[test]
    fn format_function_when_some_args_are_multiline_because_of_line_comments() {
        let src = "fn  foo ( a: i32, // comment
         b: i32
         )  { }  ";
        let expected = "fn foo(
    a: i32, // comment
    b: i32,
) {}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_when_some_args_are_multiline_because_of_line_comments_2() {
        let src = "fn  foo ( a: i32, // comment
        // another
         b: i32 // another comment
         )  { }  ";
        let expected = "fn foo(
    a: i32, // comment
    // another
    b: i32, // another comment
) {}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_when_some_args_are_multiline_because_of_block_comments() {
        let src = "fn  foo ( a: i32 /*
        some
        comment */, b: i32
         )  { }  ";
        let expected = "fn foo(
    a: i32 /*
        some
        comment */,
    b: i32,
) {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_modifiers() {
        let src = "pub  unconstrained  comptime  fn  foo ( ) {  }";
        let expected = "pub unconstrained comptime fn foo() {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_unconstrained_before_pub() {
        let src = "unconstrained pub  fn  foo ( ) {  }";
        let expected = "pub unconstrained fn foo() {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_generics() {
        let src = "fn  foo < A, B, >( ) {  }";
        let expected = "fn foo<A, B>() {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_return_type() {
        let src = "fn  foo( )  ->   Field  {  }";
        let expected = "fn foo() -> Field {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_parameter_pub_visibility() {
        let src = "fn  foo( x : pub u8 ) {  }";
        let expected = "fn foo(x: pub u8) {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_parameter_calldata_visibility() {
        let src = "fn  foo( x :  call_data ( 1 )  u8 ) {  }";
        let expected = "fn foo(x: call_data(1) u8) {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_parameter_return_data_visibility() {
        let src = "fn  foo( x :  return_data   u8 ) {  }";
        let expected = "fn foo(x: return_data u8) {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_return_visibility() {
        let src = "fn  foo( )  ->  pub   Field  {  }";
        let expected = "fn foo() -> pub Field {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_empty_where_clause() {
        let src = "mod foo { fn  foo( )  where    {  } } ";
        let expected = "mod foo {
    fn foo() {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_where_clause() {
        let src = "mod foo { fn  foo( )  where  T : Foo , U :  Bar   {  } } ";
        let expected = "mod foo {
    fn foo()
    where
        T: Foo,
        U: Bar,
    {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_where_clause_multiple_bounds() {
        let src = "mod foo { fn  foo( )  where  T : Foo+Bar , U :  Baz  +  Qux   {  } } ";
        let expected = "mod foo {
    fn foo()
    where
        T: Foo + Bar,
        U: Baz + Qux,
    {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_comment_after_parameters() {
        let src = "fn main()
        // hello 
    {}";
        let expected = "fn main()
// hello
{}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_line_comment_in_parameters() {
        let src = "fn main(
        // hello
        )
    {}";
        let expected = "fn main(
    // hello
) {}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_line_comment_on_top_of_parameter() {
        let src = "fn main(
// hello
unit: ()
) {}";
        let expected = "fn main(
    // hello
    unit: (),
) {}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_block_comment_in_params() {
        let src = "fn main(/* test */) {}";
        let expected = "fn main(/* test */) {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_body() {
        let src = "fn main() { 1; 2; 3 }";
        let expected = "fn main() {
    1;
    2;
    3
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_body_and_block_comment() {
        let src = "fn main() { 
        /* foo */ 
        1 }";
        let expected = "fn main() {
    /* foo */
    1
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_body_newlines() {
        let src = "fn main() { 

        1; 
        
        2; 
        
        3 

        }";
        let expected = "fn main() {
    1;

    2;

    3
}
";
        assert_format(src, expected);
    }
}
