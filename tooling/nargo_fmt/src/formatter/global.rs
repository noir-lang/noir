use noirc_frontend::{
    ast::{ItemVisibility, LetStatement, UnresolvedTypeData},
    token::{Keyword, Token},
};

use super::{chunks::Chunks, Formatter};

impl<'a> Formatter<'a> {
    pub(super) fn format_global(
        &mut self,
        let_statement: LetStatement,
        visibility: ItemVisibility,
    ) {
        let mut chunks = Chunks::new();
        chunks.text(self.chunk(|formatter| {
            formatter.write_indentation();
        }));

        chunks.text(self.chunk(|formatter| {
            formatter.format_item_visibility(visibility);
            formatter.write_keyword(Keyword::Global);
            formatter.write_space();
            formatter.format_pattern(let_statement.pattern);

            if let_statement.r#type.typ != UnresolvedTypeData::Unspecified {
                formatter.write_token(Token::Colon);
                formatter.write_space();
                formatter.format_type(let_statement.r#type);
            }

            formatter.write_space();
            formatter.write_token(Token::Assign);
        }));

        chunks.increase_indentation();
        chunks.space_or_line();
        self.format_expression(let_statement.expression, &mut chunks);
        chunks.text(self.chunk(|formatter| {
            formatter.write_semicolon();
        }));
        chunks.decrease_indentation();

        self.format_chunks(chunks);

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
