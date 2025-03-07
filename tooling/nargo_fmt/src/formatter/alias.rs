use noirc_frontend::{
    ast::{NoirTypeAlias, NormalTypeAlias},
    token::{Keyword, Token},
};

use super::Formatter;

impl Formatter<'_> {
    pub(super) fn format_type_alias(&mut self, type_alias: NormalTypeAlias) {
        self.write_indentation();
        self.format_item_visibility(type_alias.visibility);
        self.write_keyword(Keyword::Type);
        self.write_space();
        self.write_identifier(type_alias.name);
        self.format_generics(type_alias.generics);
        self.write_space();
        self.write_token(Token::Assign);
        self.write_space();
        self.format_type(type_alias.typ);
        self.write_semicolon();
    }

    pub(super) fn format_noir_type_alias(&mut self, noir_type_alias: NoirTypeAlias) {
        let type_alias = match noir_type_alias {
            NoirTypeAlias::NumericTypeAlias(numeric_type_alias) => numeric_type_alias.type_alias,
            NoirTypeAlias::NormalTypeAlias(normal_type_alias) => normal_type_alias,
        };
        self.format_type_alias(type_alias);
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
    fn format_generic_type_alias() {
        let src = "  pub  type  Foo < A, B > =   i32  ; ";
        let expected = "pub type Foo<A, B> = i32;\n";
        assert_format(src, expected);
    }
}
