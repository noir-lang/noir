use noirc_frontend::token::Token;

use super::{chunks::TextChunk, Formatter};

impl<'a> Formatter<'a> {
    pub(super) fn write_space(&mut self) {
        self.skip_comments_and_whitespace();
        self.write_space_without_skipping_whitespace_and_comments();
    }

    pub(super) fn write_space_without_skipping_whitespace_and_comments(&mut self) {
        if !self.buffer.ends_with('\n') && !self.buffer.ends_with(' ') {
            self.write(" ");
        }
    }

    pub(super) fn skip_whitespace_if_it_is_not_a_newline(&mut self) {
        while let Token::Whitespace(whitespace) = &self.token {
            if whitespace.contains('\n') {
                break;
            }
            self.bump();
        }
    }

    pub(super) fn skip_comments_and_whitespace(&mut self) {
        self.skip_comments_and_whitespace_impl(
            false, // write lines
            false, // at beginning
        )
    }

    pub(super) fn skip_comments_and_whitespace_writing_lines_if_found(&mut self) {
        self.skip_comments_and_whitespace_impl(
            true,  // write lines
            false, // at beginning
        )
    }

    pub(super) fn skip_comments_and_whitespace_impl(
        &mut self,
        write_lines: bool,
        at_beginning: bool,
    ) {
        let mut number_of_newlines = 0;
        let mut passed_whitespace = false;
        let mut last_was_block_comment = false;
        loop {
            match &self.token {
                Token::Whitespace(whitespace) => {
                    number_of_newlines = whitespace.chars().filter(|char| *char == '\n').count();
                    passed_whitespace = whitespace.ends_with(' ');

                    if last_was_block_comment && number_of_newlines > 0 {
                        if number_of_newlines > 1 {
                            self.write_multiple_lines_without_skipping_whitespace_and_comments();
                        } else {
                            self.write_line_without_skipping_whitespace_and_comments();
                        }

                        self.bump();

                        // Only indent for what's coming next if it's a comment
                        // (otherwise a closing brace must come and we wouldn't want to indent that)
                        if matches!(
                            &self.token,
                            Token::LineComment(_, None) | Token::BlockComment(_, None),
                        ) {
                            self.write_indentation();
                        }

                        number_of_newlines = 0;
                        passed_whitespace = false;
                    } else {
                        self.bump();
                    }

                    last_was_block_comment = false;
                }
                Token::LineComment(_, None) => {
                    if number_of_newlines > 1 && write_lines {
                        self.write_multiple_lines_without_skipping_whitespace_and_comments();
                        self.write_indentation();
                    } else if number_of_newlines > 0 {
                        self.write_line_without_skipping_whitespace_and_comments();
                        self.write_indentation();
                    } else {
                        if !(at_beginning && self.buffer.is_empty()) {
                            self.write_space_without_skipping_whitespace_and_comments();
                        }
                    }
                    self.write_current_token_trimming_end();
                    self.write_line_without_skipping_whitespace_and_comments();
                    number_of_newlines = 1;
                    self.bump();
                    passed_whitespace = false;
                    last_was_block_comment = false;
                    self.wrote_comment = true;
                }
                Token::BlockComment(_, None) => {
                    if number_of_newlines > 1 && write_lines {
                        self.write_multiple_lines_without_skipping_whitespace_and_comments();
                        self.write_indentation();
                    } else if number_of_newlines > 0 {
                        self.write_line_without_skipping_whitespace_and_comments();
                        self.write_indentation();
                    } else if passed_whitespace {
                        self.write_space_without_skipping_whitespace_and_comments();
                    }
                    self.write_current_token();
                    self.bump();
                    passed_whitespace = false;
                    last_was_block_comment = true;
                    self.wrote_comment = true;
                }
                _ => break,
            }
        }

        if number_of_newlines > 1 && write_lines {
            self.write_multiple_lines_without_skipping_whitespace_and_comments();
        }
    }

    pub(super) fn following_newlines_count(&mut self) -> usize {
        let Token::Whitespace(whitespace) = &self.token else {
            return 0;
        };

        whitespace.chars().filter(|char| *char == '\n').count()
    }

    pub(super) fn write_line(&mut self) {
        self.skip_comments_and_whitespace_impl(
            true,  // writing newline
            false, // at beginning
        );
        self.write_line_without_skipping_whitespace_and_comments();
    }

    pub(super) fn write_line_without_skipping_whitespace_and_comments(&mut self) -> bool {
        if !self.buffer.ends_with('\n') && !self.buffer.ends_with(' ') {
            self.write("\n");
            true
        } else {
            false
        }
    }

    pub(super) fn write_multiple_lines_without_skipping_whitespace_and_comments(&mut self) {
        if self.buffer.ends_with("\n\n") {
            // Nothing
        } else if self.buffer.ends_with("\n") {
            self.write("\n")
        } else {
            self.write("\n\n");
        }
    }

    pub(super) fn skip_comments_and_whitespace_chunk(&mut self) -> TextChunk {
        self.chunk(|formatter| {
            formatter.skip_comments_and_whitespace();
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_format, assert_format_with_max_width};

    #[test]
    fn format_array_in_global_with_line_comments() {
        let src = "global x = [ // hello
        1 , 2 ] ;";
        let expected = "global x = [
    // hello
    1, 2,
];
";
        assert_format(src, expected);
    }

    #[test]
    fn format_array_in_global_with_line_comments_2() {
        let src = "global x = [ // hello
         [ 1 , 2 ]  ] ;";
        let expected = "global x = [
    // hello
    [1, 2],
];
";
        assert_format(src, expected);
    }

    #[test]
    fn format_array_in_global_with_line_comments_3() {
        let src = "global x =
    [ 
        // hello
        [1, 2],  
    ];
";
        let expected = "global x = [
    // hello
    [1, 2],
];
";
        assert_format(src, expected);
    }

    #[test]
    fn format_array_in_global_with_line_comments_4() {
        let src = "global x =
    [
        1, // world 
        2, 3,
    ];
";
        let expected = "global x = [
    1, // world
    2, 3,
];
";
        assert_format(src, expected);
    }

    #[test]
    fn format_array_in_global_with_block_comments() {
        let src = "global x = [ /* hello */
        1 , 2 ] ;";
        let expected = "global x = [
    /* hello */
    1, 2,
];
";
        assert_format_with_max_width(src, expected, 20);
    }

    #[test]
    fn format_if_with_comment_after_condition() {
        let src = "global x = if  123  // some comment  
        {   456   }  ;";
        let expected = "global x = if 123 // some comment
{
    456
};
";
        assert_format(src, expected);
    }

    #[test]
    fn format_if_with_comment_after_else() {
        let src = "global x = if  123  {   456   } else  // some comment 
        { 789 };";
        let expected = "global x = if 123 {
    456
} else // some comment
{
    789
};
";
        assert_format(src, expected);
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
    fn format_function_with_body_one_expr_trailing_comment() {
        let src = "mod moo { fn main() { 1   // yes
        } }";
        let expected = "mod moo {
    fn main() {
        1 // yes
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_body_one_expr_semicolon_trailing_comment() {
        let src = "mod moo { fn main() { 1  ; // yes
        } }";
        let expected = "mod moo {
    fn main() {
        1; // yes
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_many_exprs_trailing_comments() {
        let src = "mod moo { fn main() { 1  ; // yes
        2 ; // no
        3 // maybe
        } }";
        let expected = "mod moo {
    fn main() {
        1; // yes
        2; // no
        3 // maybe
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_function_with_block_comment_after_two_newlines() {
        let src = "fn foo() {
    1;

    /* world */
    2
}
";
        let expected = "fn foo() {
    1;

    /* world */
    2
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_comment_on_top_of_let_followed_by_statement() {
        let src = "fn foo() {
    1;

    // Comment
    let x = 1;
}
";
        let expected = "fn foo() {
    1;

    // Comment
    let x = 1;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_module_declaration_with_block_comments() {
        let src = "  mod/*a*/ foo /*b*/ ; ";
        let expected = "mod/*a*/ foo /*b*/;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_module_declaration_with_inline_comments() {
        let src = "  mod // a  
 foo // b 
  ; ";
        let expected = "mod // a
foo // b
;
";
        assert_format(src, expected);
    }

    #[test]
    fn format_submodule_with_line_comments_in_separate_line() {
        let src = " #[foo] pub  mod foo { 
// one
#[hello]
mod bar; 
// two
}";
        let expected = "#[foo]
pub mod foo {
    // one
    #[hello]
    mod bar;
    // two
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_submodule_with_line_comment_in_same_line() {
        let src = " #[foo] pub  mod foo {  // one
mod bar; 
}";
        let expected = "#[foo]
pub mod foo { // one
    mod bar;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_submodule_with_block_comment() {
        let src = " #[foo] pub  mod foo {  /* one */
/* two */
mod bar; 
}";
        let expected = "#[foo]
pub mod foo { /* one */
    /* two */
    mod bar;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_submodule_with_block_comment_2() {
        let src = "mod foo {
        /* one */
}";
        let expected = "mod foo {
    /* one */
}
";
        assert_format(src, expected);
    }

    #[test]
    fn keeps_spaces_between_comments() {
        let src = "  mod  foo { 

// hello

// world

} ";
        let expected = "mod foo {

    // hello

    // world

}
";
        assert_format(src, expected);
    }

    #[test]
    fn comment_with_leading_space() {
        let src = "    // comment
        // hello
mod  foo ; ";
        let expected = "// comment
// hello
mod foo;
";
        assert_format(src, expected);
    }

    #[test]
    fn format_empty_block_statement_with_inline_block_comment() {
        let src = " fn foo() { { /* hello */ } } ";
        let expected = "fn foo() {
    { /* hello */ }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_empty_struct_with_block_comments() {
        let src = " struct Foo {
        /* hello */
    }
        ";
        let expected = "struct Foo { /* hello */ }\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_struct_with_just_comments() {
        let src = " mod foo { struct Foo {
// hello
    } }
        ";
        let expected = "mod foo {
    struct Foo {
        // hello
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_block_comment_no_whitespace_in_block_single_line() {
        let src = "global x = {/*foo*/};";
        let expected = "global x = { /*foo*/ };\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_block_comment_no_whitespace_but_newline_in_block_single_line() {
        let src = "global x = {/*foo*/
        };";
        let expected = "global x = { /*foo*/ };\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_line_comment_in_block_same_line() {
        let src = "global x = {       // foo
        };";
        let expected = "global x = { // foo
};
";
        assert_format(src, expected);
    }

    #[test]
    fn format_line_comment_in_block_separate_line() {
        let src = "global x = {
        // foo
        };";
        let expected = "global x = {
    // foo
};
";
        assert_format(src, expected);
    }

    #[test]
    fn format_block_comment_in_parenthesized_expression() {
        let src = "global x = ( /* foo */ 1 );";
        let expected = "global x = ( /* foo */ 1);\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_line_comment_in_parenthesized() {
        let src = "global x = ( // hello 
        1 );";
        let expected = "global x = (
    // hello
    1
);\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_index_with_comment() {
        let src = "global x = foo[// hello
        1];";
        let expected = "global x = foo[
    // hello
    1
];\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_comment_in_infix_between_lhs_and_operator() {
        let src = "global x = 1/* comment */+ 2 ;";
        let expected = "global x = 1 /* comment */ + 2;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_comment_in_constructor_inside_function() {
        let src = "fn foo() { MyStruct {/*test*/}; } ";
        let expected = "fn foo() {
    MyStruct { /*test*/ };
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_block_comment_before_constructor_field() {
        let src = "global x = Foo {/*comment*/field}; ";
        let expected = "global x = Foo { /*comment*/ field };\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_line_comment_before_constructor_field() {
        let src = "global x = Foo { // foo
        field}; ";
        let expected = "global x = Foo {
    // foo
    field,
};\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_comment_in_empty_constructor() {
        let src = "global x = Foo { // comment
        }; ";
        let expected = "global x = Foo { // comment
};\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_comment_after_parenthesized() {
        let src = "global x = (
            1
            // hello
        )
        ; ";
        let expected = "global x = (
    1
    // hello
);\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_comment_in_single_element_tuple() {
        let src = "global x = ( 1 /* hello */ , );";
        let expected = "global x = (1 /* hello */,);\n";
        assert_format(src, expected);
    }
}
