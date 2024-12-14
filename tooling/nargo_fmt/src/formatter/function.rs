use noirc_frontend::{
    ast::{
        BlockExpression, FunctionReturnType, Ident, ItemVisibility, NoirFunction, Param,
        UnresolvedGenerics, UnresolvedTraitConstraint, Visibility,
    },
    token::{Attributes, Keyword, Token},
};

use super::Formatter;
use crate::chunks::{ChunkGroup, TextChunk};

pub(super) struct FunctionToFormat {
    pub(super) attributes: Attributes,
    pub(super) visibility: ItemVisibility,
    pub(super) name: Ident,
    pub(super) generics: UnresolvedGenerics,
    pub(super) parameters: Vec<Param>,
    pub(super) return_type: FunctionReturnType,
    pub(super) return_visibility: Visibility,
    pub(super) where_clause: Vec<UnresolvedTraitConstraint>,
    pub(super) body: Option<BlockExpression>,
}

impl<'a> Formatter<'a> {
    pub(super) fn format_function(&mut self, func: NoirFunction) {
        self.format_function_impl(FunctionToFormat {
            attributes: func.def.attributes,
            visibility: func.def.visibility,
            name: func.def.name,
            generics: func.def.generics,
            parameters: func.def.parameters,
            return_type: func.def.return_type,
            return_visibility: func.def.return_visibility,
            where_clause: func.def.where_clause,
            body: Some(func.def.body),
        });
    }

    pub(super) fn format_function_impl(&mut self, func: FunctionToFormat) {
        let has_where_clause = !func.where_clause.is_empty();

        self.format_attributes(func.attributes);
        self.write_indentation();
        self.format_function_modifiers(func.visibility);
        self.write_keyword(Keyword::Fn);
        self.write_space();
        self.write_identifier(func.name);
        self.format_generics(func.generics);
        self.write_left_paren();

        // When the function has no parameters we can format everything in a single line
        if func.parameters.is_empty() {
            self.increase_indentation();
            self.skip_comments_and_whitespace();
            self.decrease_indentation();

            let group = ChunkGroup::new();
            self.format_function_right_paren_until_left_brace_or_semicolon(
                func.return_type,
                func.return_visibility,
                has_where_clause,
                false,               // has parameters
                func.body.is_none(), // semicolon
                group,
            );
        } else {
            let mut group = ChunkGroup::new();
            self.format_function_parameters(func.parameters, &mut group);
            self.format_function_right_paren_until_left_brace_or_semicolon(
                func.return_type,
                func.return_visibility,
                has_where_clause,
                true,                // has parameters
                func.body.is_none(), // semicolon
                group,
            );
        }

        if has_where_clause {
            self.format_where_clause(
                func.where_clause,
                func.body.is_some(), // write trailing comma and newline
            );
            if func.body.is_some() {
                self.write_left_brace();
            } else {
                self.write_semicolon();
            }
        }

        if let Some(body) = func.body {
            self.format_function_body(body);
            self.write_right_brace();
        }
    }

    pub(super) fn format_function_modifiers(&mut self, visibility: ItemVisibility) {
        // For backwards compatibility, unconstrained might come before visibility.
        // We'll remember this but put it after the visibility.
        let unconstrained = if self.is_at_keyword(Keyword::Unconstrained) {
            self.bump();
            self.skip_comments_and_whitespace();
            true
        } else {
            false
        };

        self.format_item_visibility(visibility);

        if unconstrained {
            self.write("unconstrained ");
        } else if self.is_at_keyword(Keyword::Unconstrained) {
            self.write_keyword(Keyword::Unconstrained);
            self.write_space();
        }

        if self.is_at_keyword(Keyword::Comptime) {
            self.write_keyword(Keyword::Comptime);
            self.write_space();
        }
    }

    pub(super) fn format_function_parameters(
        &mut self,
        parameters: Vec<Param>,
        group: &mut ChunkGroup,
    ) {
        self.chunk_formatter().format_items_separated_by_comma(
            parameters,
            false, // force trailing comma
            false, // surround with spaces
            group,
            |formatter, param, group| {
                group.text(formatter.chunk(|formatter| {
                    formatter.format_function_param(param);
                }));
            },
        );
    }

    fn format_function_param(&mut self, param: Param) {
        self.format_pattern(param.pattern);
        self.skip_comments_and_whitespace();

        // There might not be a colon if the parameter is self
        if self.is_at(Token::Colon) {
            self.write_token(Token::Colon);
            self.write_space();
            self.format_visibility(param.visibility);
            self.format_type(param.typ);
        }
    }

    /// Returns whether the left brace of semicolon was written
    /// (we don't write it when there's a comment before those tokens)
    pub(super) fn format_function_right_paren_until_left_brace_or_semicolon(
        &mut self,
        return_type: FunctionReturnType,
        visibility: Visibility,
        has_where_clause: bool,
        has_parameters: bool,
        semicolon: bool,
        mut group: ChunkGroup,
    ) {
        let mut chunk_formatter = self.chunk_formatter();

        group.text(chunk_formatter.chunk(|formatter| {
            formatter.write_right_paren();
            formatter.format_function_return_type(return_type, visibility);
        }));

        // The following code is a bit long because it takes into account three scenarios:
        //
        // 1.
        // fn foo() -> Field {}
        //
        // 2.
        // fn foo() -> Field // comment
        // {}
        //
        // 3.
        // fn foo() -> Field
        // // comment
        // {}
        //
        // We want to preserve the above formatting when there are trailing comments,
        // possibly considering a trailing comment in the same line to count towards the
        // maximum width of the line.
        //
        // For that, we take the comment chunk and depending on whether it has leading newlines
        // or if it even exists we take different paths.
        let comment_chunk = chunk_formatter.skip_comments_and_whitespace_chunk();
        let comment_chunk = TextChunk::new(comment_chunk.string.trim_end().to_string());

        let comment_starts_with_newline = comment_chunk.string.trim_matches(' ').starts_with('\n');
        if comment_starts_with_newline {
            // After the return type we found a newline and a comment. We want to format the group
            // right away, then keep formatting everything else (at that point there's no need to
            // use chunks anymore).
            self.format_chunk_group(group);

            let mut comment_group = ChunkGroup::new();
            comment_group.text(comment_chunk);
            self.format_chunk_group(comment_group);
            self.write_line();

            // If there's no where clause the left brace goes on the same line as the function signature
            if !has_where_clause {
                self.skip_stray_where_keyword();

                if semicolon {
                    self.write_semicolon();
                } else {
                    self.write_left_brace();
                }
            }
            return;
        }

        let wrote_comment = !comment_chunk.string.trim().is_empty();
        group.text(comment_chunk);

        // If there's no where clause the left brace goes on the same line as the function signature
        if !has_where_clause {
            group.text(chunk_formatter.chunk(|formatter| {
                formatter.skip_stray_where_keyword();
            }));
        }

        if !has_where_clause && !wrote_comment {
            if semicolon {
                group.semicolon(&mut chunk_formatter);
            } else {
                group.text(chunk_formatter.chunk(|formatter| {
                    formatter.write_space();
                    formatter.write_left_brace();
                }));
            }
        }

        if has_parameters {
            self.format_chunk_group(group);
        } else {
            self.format_chunk_group_in_one_line(group);
        }

        if wrote_comment {
            self.write_line();
            if semicolon {
                self.write_semicolon();
            } else {
                self.write_left_brace();
            }
        }
    }

    fn skip_stray_where_keyword(&mut self) {
        // There might still be a where keyword that we'll remove
        if self.is_at_keyword(Keyword::Where) {
            self.bump();
            self.skip_comments_and_whitespace();
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

    pub(super) fn format_function_body(&mut self, body: BlockExpression) {
        let mut group = ChunkGroup::new();
        self.chunk_formatter().format_block_expression_contents(
            body, true, // force multiple newlines
            &mut group,
        );
        self.format_chunk_group(group);
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

    #[test]
    fn format_function_with_body_one_expr() {
        let src = "mod moo { fn main() { 1 } }";
        let expected = "mod moo {
    fn main() {
        1
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_empty_body_multiple_lines() {
        let src = "fn foo() {

        }";
        let expected = "fn foo() {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_trailing_comment_in_same_line_before_left_brace() {
        let src = "fn foo(x: Field) // comment 
        {
        }";
        let expected = "fn foo(x: Field) // comment
{}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_trailing_comment_in_separate_line_before_left_brace() {
        let src = "fn foo(x: Field)
        // comment 
        {
        }";
        let expected = "fn foo(x: Field)
// comment
{}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_long_function_signature_no_parameters() {
        let src = "fn foo() -> Field {}";
        let expected = "fn foo() -> Field {}\n";
        assert_format_with_max_width(src, expected, 15);
    }

    #[test]
    fn does_not_format_function_if_there_is_a_directive_not_to() {
        let src = "// noir-fmt:ignore
fn foo() { let  x  = 1  ; 
            }
                   
fn bar() { let  y  = 2  ; 
            }

// noir-fmt:ignore
fn baz() { let  z  = 3  ; 
            }

";
        let expected = "// noir-fmt:ignore
fn foo() { let  x  = 1  ; 
            }

fn bar() {
    let y = 2;
}

// noir-fmt:ignore
fn baz() { let  z  = 3  ; 
            }

";
        assert_format(src, expected);
    }

    #[test]
    fn comment_in_body_respects_newlines() {
        let src = "fn foo() {
    let x = 1;

    // comment

    let y = 2;
}
";
        let expected = src;
        assert_format(src, expected);
    }

    #[test]
    fn final_comment_in_body_respects_newlines() {
        let src = "fn foo() {
    let x = 1;

    let y = 2;

    // comment
}
";
        let expected = src;
        assert_format(src, expected);
    }

    #[test]
    fn initial_comment_in_body_respects_newlines() {
        let src = "fn foo() {
    // comment

    let x = 1;

    let y = 2;
}
";
        let expected = src;
        assert_format(src, expected);
    }

    #[test]
    fn keeps_newlines_between_comments_no_statements() {
        let src = "fn foo() {
    // foo

    // bar

    // baz
}
";
        let expected = src;
        assert_format(src, expected);
    }

    #[test]
    fn keeps_newlines_between_comments_one_statement() {
        let src = "fn foo() {
    let x = 1;

    // foo

    // bar

    // baz
}
";
        let expected = src;
        assert_format(src, expected);
    }
}
