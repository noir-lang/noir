use noirc_frontend::{
    ast::NoirStruct,
    token::{Keyword, Token},
};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_struct(&mut self, noir_struct: NoirStruct) {
        if !noir_struct.attributes.is_empty() {
            self.format_attributes();
        }
        self.write_indentation();
        self.format_item_visibility(noir_struct.visibility);
        self.write_keyword(Keyword::Struct);
        self.write_space();
        self.write_identifier(noir_struct.name);
        self.format_generics(noir_struct.generics);
        self.skip_comments_and_whitespace();

        // A case like `struct Foo;`
        if self.token == Token::Semicolon {
            self.write_semicolon();
            return;
        }

        // A case like `struct Foo { ... }`
        self.write_space();
        self.write_left_brace();
        self.increase_indentation();
        self.write_line();

        for documented_field in noir_struct.fields {
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
            self.skip_comments_and_whitespace();

            if self.token == Token::Comma {
                self.bump();
            }
            self.write(",");
            self.write_line();
        }

        self.write_line();
        self.deincrease_indentation();
        self.write_indentation();
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
}
