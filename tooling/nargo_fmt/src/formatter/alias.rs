use noirc_frontend::{
    ast::TypeAlias,
    token::{Keyword, Token},
};

use super::Formatter;

impl Formatter<'_> {
    pub(super) fn format_type_alias(&mut self, type_alias: TypeAlias) {
        self.write_indentation();
        self.format_item_visibility(type_alias.visibility);
        self.write_keyword(Keyword::Type);
        self.write_space();
        self.write_identifier(type_alias.name);
        self.format_generics(type_alias.generics);
        if let Some(num_type) = type_alias.numeric_type {
            self.write_token(Token::Colon);
            self.write_space();
            self.format_type(num_type);
        }
        self.write_space();
        self.write_token(Token::Assign);
        self.write_space();
        self.format_type(type_alias.typ);
        self.write_semicolon();
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_format;

    #[test]
    fn format_type_alias() {
        let src = "  pub  type  Foo  =   i32  ; ";
        let expected = "pub type Foo = i32;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_num_type_alias() {
        let src = "  pub  type  Foo:u32  =  2*N  ; ";
        let expected = "pub type Foo: u32 = 2 * N;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_generic_type_alias() {
        let src = "  pub  type  Foo < A, B > =   i32  ; ";
        let expected = "pub type Foo<A, B> = i32;\n";
        assert_format(src, expected);
    }
}
