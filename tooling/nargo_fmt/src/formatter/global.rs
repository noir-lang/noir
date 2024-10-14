use noirc_frontend::{
    ast::{ItemVisibility, LetStatement, UnresolvedTypeData},
    token::{Keyword, Token},
};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_global(
        &mut self,
        let_statement: LetStatement,
        visibility: ItemVisibility,
    ) {
        self.write_indentation();
        self.format_item_visibility(visibility);
        self.write_keyword(Keyword::Global);
        self.write_space();
        self.format_pattern(let_statement.pattern);

        if let_statement.r#type.typ != UnresolvedTypeData::Unspecified {
            self.write_token(Token::Colon);
            self.write_space();
            self.format_type(let_statement.r#type);
        }

        self.write_space();
        self.write_token(Token::Assign);
        self.write_space();
        self.format_expression(let_statement.expression);
        self.write_semicolon();
        self.write_line();
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_format;

    #[test]
    fn format_global_without_type() {
        let src = " pub  global  x  =  1  ; ";
        let expected = "pub global x = 1;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_global_with_type() {
        let src = " pub  global  x  :  Field  =  1  ; ";
        let expected = "pub global x: Field = 1;\n";
        assert_format(src, expected);
    }
}
