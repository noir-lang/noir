use noirc_frontend::{
    ast::{FunctionReturnType, NoirFunction},
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

        // For backwards compatibility, unconstrained might come before visibility.
        // We'll remember this but put it after the visibility.
        let unconstrained = if self.token == Token::Keyword(Keyword::Unconstrained) {
            self.bump();
            self.skip_comments_and_whitespace();
            true
        } else {
            false
        };

        self.format_item_visibility(func.def.visibility);

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

        self.write_keyword(Keyword::Fn);
        self.write_space();
        self.write_identifier(func.def.name);
        self.format_generics(func.def.generics);
        self.write_left_paren();

        let mut chunks = Chunks::new();
        chunks.increase_indentation();
        chunks.line();

        for (index, param) in func.def.parameters.into_iter().enumerate() {
            if index > 0 {
                chunks.text(self.chunk(|formatter| {
                    formatter.write_comma();
                    formatter.write_space();
                }));
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

        chunks.decrease_indentation();
        chunks.line();

        chunks.text(self.chunk(|formatter| {
            formatter.write_right_paren();
            formatter.write_space();

            match func.def.return_type {
                FunctionReturnType::Default(..) => (),
                FunctionReturnType::Ty(typ) => {
                    formatter.write_token(Token::Arrow);
                    formatter.write_space();
                    formatter.format_visibility(func.def.return_visibility);
                    formatter.format_type(typ);
                    formatter.write_space();
                }
            }

            formatter.write_left_brace();
        }));

        self.format_chunks(chunks);

        self.write_right_brace();
        self.write_line();
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
         b: i32 // another comment
         )  { }  ";
        let expected = "fn foo(
    a: i32, // comment
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
}
