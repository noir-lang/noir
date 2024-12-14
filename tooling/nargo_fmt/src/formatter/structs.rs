use noirc_frontend::{
    ast::NoirStruct,
    token::{Keyword, Token},
};

use super::Formatter;
use crate::chunks::ChunkGroup;

impl<'a> Formatter<'a> {
    pub(super) fn format_struct(&mut self, noir_struct: NoirStruct) {
        self.format_secondary_attributes(noir_struct.attributes);
        self.write_indentation();
        self.format_item_visibility(noir_struct.visibility);
        self.write_keyword(Keyword::Struct);
        self.write_space();
        self.write_identifier(noir_struct.name);
        self.format_generics(noir_struct.generics);
        self.skip_comments_and_whitespace();

        // A case like `struct Foo;`
        if self.is_at(Token::Semicolon) {
            self.write_semicolon();
            return;
        }

        // A case like `struct Foo { ... }`
        self.write_space();
        self.write_left_brace();

        if noir_struct.fields.is_empty() {
            self.format_empty_block_contents();
        } else {
            self.increase_indentation();
            self.write_line();

            for (index, documented_field) in noir_struct.fields.into_iter().enumerate() {
                if index > 0 {
                    self.write_comma();
                    self.write_line();
                }

                let doc_comments = documented_field.doc_comments;
                if !doc_comments.is_empty() {
                    self.format_outer_doc_comments();
                }

                let field = documented_field.item;
                self.write_indentation();
                self.format_item_visibility(field.visibility);
                self.write_identifier(field.name);
                self.write_token(Token::Colon);
                self.write_space();
                self.format_type(field.typ);
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
    fn format_empty_struct_semicolon() {
        let src = " mod moo { 
    /// hello
    #[foo] pub ( crate ) struct Foo  ; }";
        let expected = "mod moo {
    /// hello
    #[foo]
    pub(crate) struct Foo;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_empty_struct_with_generics() {
        let src = " mod moo { struct Foo < A, B, let N : u32  > ; }";
        let expected = "mod moo {
    struct Foo<A, B, let N: u32>;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_struct_with_fields() {
        let src = " mod moo { struct Foo { 
// hello
/// comment
  pub field  : Field  ,
  // comment
pub another : ( ),
        } }";
        let expected = "mod moo {
    struct Foo {
        // hello
        /// comment
        pub field: Field,
        // comment
        pub another: (),
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_struct_with_multiple_newlines() {
        let src = " mod moo { 


    struct Foo { 


x: Field  ,


y: Field


} 


}";
        let expected = "mod moo {

    struct Foo {

        x: Field,

        y: Field,
    }

}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_two_structs() {
        let src = " struct Foo { } struct Bar {}
        ";
        let expected = "struct Foo {}
struct Bar {}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_struct_field_without_trailing_comma_but_comment() {
        let src = "struct Foo {
    field: Field // comment
        }";
        let expected = "struct Foo {
    field: Field, // comment
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_comment_after_last_struct_field() {
        let src = "struct Foo {
    field: Field 
    /* comment */
        }";
        let expected = "struct Foo {
    field: Field,
    /* comment */
}
";
        assert_format(src, expected);
    }
}
