use noirc_frontend::{
    ast::{Ident, Pattern},
    token::{Keyword, Token},
};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_pattern(&mut self, pattern: Pattern) {
        self.skip_comments_and_whitespace();

        // Special case: `&mut self` (this is reflected in the param type, not the pattern)
        if self.token == Token::Ampersand {
            self.write_token(Token::Ampersand);
            self.write_keyword(Keyword::Mut);
            self.write_space();
        }

        match pattern {
            Pattern::Identifier(ident) => self.write_identifier(ident),
            Pattern::Mutable(pattern, _span, _) => {
                self.write_keyword(Keyword::Mut);
                self.write_space();
                self.format_pattern(*pattern);
            }
            Pattern::Tuple(patterns, _span) => {
                let patterns_len = patterns.len();

                self.write_left_paren();
                for (index, pattern) in patterns.into_iter().enumerate() {
                    if index > 0 {
                        self.write_comma();
                        self.write_space();
                    }
                    self.format_pattern(pattern);
                }

                // Check for trailing comma
                self.skip_comments_and_whitespace();
                if self.token == Token::Comma {
                    if patterns_len == 1 {
                        self.write_comma();
                    } else {
                        self.bump();
                    }
                }

                self.write_right_paren();
            }
            Pattern::Struct(path, fields, _span) => {
                self.format_path(path);
                self.write_space();
                self.write_left_brace();
                self.skip_comments_and_whitespace();
                for (index, (name, pattern)) in fields.into_iter().enumerate() {
                    let is_identifier_pattern = is_identifier_pattern(&pattern, &name);

                    if index > 0 {
                        self.write_comma();
                        self.write_space();
                    }

                    self.write_identifier(name);
                    self.skip_comments_and_whitespace();
                    if self.token == Token::Colon {
                        if is_identifier_pattern {
                            self.chunk(|formatter| {
                                formatter.write_token(Token::Colon);
                                formatter.write_space();
                                formatter.format_pattern(pattern);
                            });
                        } else {
                            self.write_token(Token::Colon);
                            self.write_space();
                            self.format_pattern(pattern);
                        }
                    }
                }
                self.write_right_brace();
            }
            Pattern::Interned(..) => {
                unreachable!("Should not be present in the AST")
            }
        }
    }
}

fn is_identifier_pattern(pattern: &Pattern, ident: &Ident) -> bool {
    if let Pattern::Identifier(pattern_ident) = pattern {
        pattern_ident == ident
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_format;

    #[test]
    fn format_identifier_pattern() {
        let src = "fn foo( x : i32) {}";
        let expected = "fn foo(x: i32) {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_mutable_pattern() {
        let src = "fn foo( mut x : i32) {}";
        let expected = "fn foo(mut x: i32) {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_tuple_pattern_no_trailing_comma() {
        let src = "fn foo( (  x  ,  y  ) : i32) {}";
        let expected = "fn foo((x, y): i32) {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_tuple_pattern_trailing_comma() {
        let src = "fn foo( (  x  ,  y , ) : i32) {}";
        let expected = "fn foo((x, y): i32) {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_tuple_pattern_one_element() {
        let src = "fn foo( (  x  ,    ) : i32) {}";
        let expected = "fn foo((x,): i32) {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_struct_pattern_empty() {
        let src = "fn foo( Foo {  } : i32) {}";
        let expected = "fn foo(Foo {}: i32) {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_struct_pattern() {
        let src = "fn foo( Foo { x : one , y : two } : i32) {}";
        let expected = "fn foo(Foo {x: one, y: two}: i32) {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_struct_pattern_no_pattern() {
        let src = "fn foo( Foo { x  , y : y } : i32) {}";
        let expected = "fn foo(Foo {x, y}: i32) {}\n";
        assert_format(src, expected);
    }
}
