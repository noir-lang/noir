use noirc_frontend::{
    ast::NoirEnumeration,
    token::{Keyword, Token},
};

use super::Formatter;
use crate::chunks::ChunkGroup;

impl<'a> Formatter<'a> {
    pub(super) fn format_enum(&mut self, noir_enum: NoirEnumeration) {
        self.format_secondary_attributes(noir_enum.attributes);
        self.write_indentation();
        self.format_item_visibility(noir_enum.visibility);
        self.write_keyword(Keyword::Enum);
        self.write_space();
        self.write_identifier(noir_enum.name);
        self.format_generics(noir_enum.generics);
        self.skip_comments_and_whitespace();

        // A case like `enum Foo;`
        if self.is_at(Token::Semicolon) {
            self.write_semicolon();
            return;
        }

        // A case like `enum Foo { ... }`
        self.write_space();
        self.write_left_brace();

        if noir_enum.variants.is_empty() {
            self.format_empty_block_contents();
        } else {
            self.increase_indentation();
            self.write_line();

            for (index, documented_variant) in noir_enum.variants.into_iter().enumerate() {
                if index > 0 {
                    self.write_comma();
                    self.write_line();
                }

                let doc_comments = documented_variant.doc_comments;
                if !doc_comments.is_empty() {
                    self.format_outer_doc_comments();
                }

                let variant = documented_variant.item;
                self.write_indentation();
                self.write_identifier(variant.name);

                if !variant.parameters.is_empty() {
                    self.write_token(Token::LeftParen);
                    for (i, parameter) in variant.parameters.into_iter().enumerate() {
                        if i != 0 {
                            self.write_comma();
                            self.write_space();
                        }
                        self.format_type(parameter);
                    }
                    self.write_token(Token::RightParen);
                } else {
                    // Remove `()` from an empty `Variant()`
                    self.skip_comments_and_whitespace();
                    if self.is_at(Token::LeftParen) {
                        self.bump();
                    }
                    self.skip_comments_and_whitespace();
                    if self.is_at(Token::RightParen) {
                        self.bump();
                    }
                }
            }

            // Take the comment chunk so we can put it after a trailing comma we add, in case there's no comma
            let mut group = ChunkGroup::new();
            let mut comments_and_whitespace_chunk =
                self.chunk_formatter().skip_comments_and_whitespace_chunk();
            comments_and_whitespace_chunk.string =
                comments_and_whitespace_chunk.string.trim_end().to_string();
            group.text(comments_and_whitespace_chunk);

            if self.is_at(Token::Comma) {
                self.bump();
            }
            self.write(",");

            self.format_chunk_group(group);
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
    fn format_empty_enum_with_generics() {
        let src = " mod moo { enum Foo < A, B, let N : u32  > {} }";
        let expected = "mod moo {
    enum Foo<A, B, let N: u32> {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_enum_with_variants() {
        let src = " mod moo { enum Foo { 
// hello
/// comment
  Variant  ( Field  ,  i32    )  ,
  // comment
 Another ( ),
        } }";
        let expected = "mod moo {
    enum Foo {
        // hello
        /// comment
        Variant(Field, i32),
        // comment
        Another,
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_enum_with_multiple_newlines() {
        let src = " mod moo { 


    enum Foo { 


X( Field)  ,


Y ( Field )


} 


}";
        let expected = "mod moo {

    enum Foo {

        X(Field),

        Y(Field),
    }

}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_two_enums() {
        let src = " enum Foo { } enum Bar {}
        ";
        let expected = "enum Foo {}
enum Bar {}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_enum_field_without_trailing_comma_but_comment() {
        let src = "enum Foo {
    field(Field) // comment
        }";
        let expected = "enum Foo {
    field(Field), // comment
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_comment_after_last_enum_field() {
        let src = "enum Foo {
    field(Field) 
    /* comment */
        }";
        let expected = "enum Foo {
    field(Field),
    /* comment */
}
";
        assert_format(src, expected);
    }
}
