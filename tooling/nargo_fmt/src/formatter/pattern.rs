use noirc_frontend::{
    ast::{Ident, Pattern},
    token::{Keyword, Token},
};

use super::Formatter;
use crate::chunks::ChunkGroup;

impl<'a> Formatter<'a> {
    pub(super) fn format_pattern(&mut self, pattern: Pattern) {
        self.skip_comments_and_whitespace();

        // Special case: `&mut self` (this is reflected in the param type, not the pattern)
        if self.is_at(Token::Ampersand) {
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
                if self.is_at(Token::Comma) {
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
                if fields.is_empty() {
                    self.format_empty_block_contents();
                } else {
                    let mut group = ChunkGroup::new();
                    self.chunk_formatter().format_items_separated_by_comma(
                        fields,
                        false, // force trailing comma,
                        true,  // surround with spaces
                        &mut group,
                        |formatter, (name, pattern), chunks| {
                            let is_identifier_pattern = is_identifier_pattern(&pattern, &name);

                            chunks.text(formatter.chunk(|formatter| {
                                formatter.write_identifier(name);
                                formatter.skip_comments_and_whitespace();
                            }));
                            if formatter.is_at(Token::Colon) {
                                let value_chunk = formatter.chunk(|formatter| {
                                    formatter.write_token(Token::Colon);
                                    formatter.write_space();
                                    formatter.format_pattern(pattern);
                                });
                                if !is_identifier_pattern {
                                    chunks.text(value_chunk);
                                }
                            }
                        },
                    );
                    self.format_chunk_group(group);
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
        let expected = "fn foo(Foo { x: one, y: two }: i32) {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_struct_pattern_no_pattern() {
        let src = "fn foo( Foo { x  , y : y } : i32) {}";
        let expected = "fn foo(Foo { x, y }: i32) {}\n";
        assert_format(src, expected);
    }
}
