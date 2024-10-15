use noirc_frontend::{
    ast::{Statement, StatementKind},
    token::Keyword,
};

use super::{chunks::Chunks, Formatter};

impl<'a> Formatter<'a> {
    pub(super) fn format_statement(&mut self, statement: Statement, mut chunks: &mut Chunks) {
        chunks.leading_comment(self.skip_comments_and_whitespace_chunk());

        match statement.kind {
            StatementKind::Let(_let_statement) => todo!("Format let statement"),
            StatementKind::Constrain(_constrain_statement) => todo!("Format constrain statement"),
            StatementKind::Expression(expression) => {
                self.format_expression(expression, &mut chunks);
            }
            StatementKind::Assign(_assign_statement) => todo!("Format assign statement"),
            StatementKind::For(_for_loop_statement) => todo!("Format for loop statement"),
            StatementKind::Break => {
                chunks.text(self.chunk(|formatter| {
                    formatter.write_keyword(Keyword::Break);
                    formatter.write_semicolon();
                }));
            }
            StatementKind::Continue => {
                chunks.text(self.chunk(|formatter| {
                    formatter.write_keyword(Keyword::Continue);
                    formatter.write_semicolon();
                }));
            }
            StatementKind::Comptime(_statement) => todo!("Format comptime statement"),
            StatementKind::Semi(expression) => {
                self.format_expression(expression, &mut chunks);

                chunks.text(self.chunk(|formatter| {
                    formatter.skip_comments_and_whitespace();
                    formatter.write_semicolon();
                }));
            }
            StatementKind::Interned(..) | StatementKind::Error => {
                unreachable!("Should not be present in the AST")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_format;

    #[test]
    fn format_expression_statement() {
        let src = " fn foo() { 1 } ";
        let expected = "fn foo() {
    1
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_semi_statement() {
        let src = " fn foo() { 1 ; } ";
        let expected = "fn foo() {
    1;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_break_statement() {
        let src = " fn foo() { break  ; } ";
        let expected = "fn foo() {
    break;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_continue_statement() {
        let src = " fn foo() { continue  ; } ";
        let expected = "fn foo() {
    continue;
}
";
        assert_format(src, expected);
    }
}
