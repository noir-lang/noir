use noirc_frontend::{
    ast::{Ident, Pattern},
    token::{Keyword, Token},
};

use crate::chunks::{ChunkFormatter, ChunkGroup};

use super::Formatter;

impl Formatter<'_> {
    #[must_use]
    pub(super) fn format_pattern(&mut self, pattern: Pattern) -> ChunkGroup {
        self.chunk_formatter().format_pattern(pattern)
    }
}

impl ChunkFormatter<'_, '_> {
    #[must_use]
    pub(super) fn format_pattern(&mut self, pattern: Pattern) -> ChunkGroup {
        let mut group = ChunkGroup::new();

        group.text(self.chunk(|formatter| {
            formatter.skip_comments_and_whitespace();

            // Special case: `&mut self` (this is reflected in the param type, not the pattern)
            if formatter.is_at(Token::Ampersand) {
                formatter.write_token(Token::Ampersand);
                formatter.write_keyword(Keyword::Mut);
                formatter.write_space();
            }
        }));

        match pattern {
            Pattern::Identifier(ident) => {
                group.text(self.chunk(|formatter| {
                    formatter.write_identifier(ident);
                }));
            }
            Pattern::Mutable(pattern, _span, _) => {
                group.text(self.chunk(|formatter| {
                    formatter.write_keyword(Keyword::Mut);
                    formatter.write_space();
                }));
                group.group(self.format_pattern(*pattern));
            }
            Pattern::Tuple(patterns, _span) => {
                group.text(self.chunk(|formatter| {
                    let patterns_len = patterns.len();

                    formatter.write_left_paren();
                    for (index, pattern) in patterns.into_iter().enumerate() {
                        if index > 0 {
                            formatter.write_comma();
                            formatter.write_space();
                        }
                        let group = formatter.format_pattern(pattern);
                        formatter.format_chunk_group(group);
                    }

                    // Check for trailing comma
                    formatter.skip_comments_and_whitespace();
                    if formatter.is_at(Token::Comma) {
                        if patterns_len == 1 {
                            formatter.write_comma();
                        } else {
                            formatter.bump();
                        }
                    }

                    formatter.write_right_paren();
                }));
            }
            Pattern::Struct(path, fields, _span) => {
                let mut inner_group = ChunkGroup::new();

                inner_group.text(self.chunk(|formatter| {
                    formatter.format_path(path);
                    formatter.write_space();
                    formatter.write_left_brace();
                }));

                if fields.is_empty() {
                    if let Some(empty_group) = self.empty_block_contents_chunk() {
                        inner_group.group(empty_group);
                    }
                } else {
                    self.format_items_separated_by_comma(
                        fields,
                        false, // force trailing comma,
                        true,  // surround with spaces
                        &mut inner_group,
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
                                    let pattern_group = formatter.format_pattern(pattern);
                                    formatter.format_chunk_group(pattern_group);
                                });
                                if !is_identifier_pattern {
                                    chunks.text(value_chunk);
                                }
                            }
                        },
                    );
                }

                inner_group.text(self.chunk(|formatter| {
                    formatter.write_right_brace();
                }));

                group.group(inner_group);
            }
            Pattern::Parenthesized(pattern, _) => {
                group.text(self.chunk(|formatter| {
                    formatter.write_left_paren();
                }));
                group.group(self.format_pattern(*pattern));
                group.text(self.chunk(|formatter| {
                    formatter.write_right_paren();
                }));
            }
            Pattern::Interned(..) => {
                unreachable!("Should not be present in the AST")
            }
        }

        group
    }
}

fn is_identifier_pattern(pattern: &Pattern, ident: &Ident) -> bool {
    if let Pattern::Identifier(pattern_ident) = pattern { pattern_ident == ident } else { false }
}

#[cfg(test)]
mod tests {
    use crate::{assert_format, assert_format_with_max_width};

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
    fn format_parenthesized_pattern_one_element() {
        let src = "fn foo( (  x      ) : i32) {}";
        let expected = "fn foo((x): i32) {}\n";
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

    #[test]
    fn format_struct_pattern_that_exceeds_max_width_when_not_deeply_nested() {
        let src = "
        fn foo() {
            let SomeStruct { one, two } = 1; 
        }
        ";
        let expected = "fn foo() {
    let SomeStruct {
        one,
        two,
    } = 1;
}
";
        assert_format_with_max_width(src, expected, 20);
    }

    #[test]
    fn format_struct_pattern_that_exceeds_max_width_when_deeply_nested() {
        let src = "
        fn foo() {
            if true {
                let SomeStruct { one, two } = 1; 
            }
        }
        ";
        let expected = "fn foo() {
    if true {
        let SomeStruct {
            one,
            two,
        } = 1;
    }
}
";
        assert_format_with_max_width(src, expected, 20);
    }
}
