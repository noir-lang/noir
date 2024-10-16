use noirc_frontend::{
    ast::{ItemVisibility, LetStatement},
    token::Keyword,
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
            formatter.format_item_visibility(visibility);
        }));
        chunks.group(self.format_let_or_global(
            Keyword::Global,
            let_statement.pattern,
            let_statement.r#type,
            Some(let_statement.expression),
        ));

        self.write_indentation();
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
