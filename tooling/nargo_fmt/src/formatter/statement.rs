use noirc_frontend::ast::{Statement, StatementKind};

use super::{chunks::Chunks, Formatter};

impl<'a> Formatter<'a> {
    pub(super) fn format_statement(&mut self, statement: Statement, mut chunks: &mut Chunks) {
        chunks.leading_comment(self.skip_comments_and_whitespace_chunk());

        match statement.kind {
            StatementKind::Let(_let_statement) => todo!("Format let statement"),
            StatementKind::Constrain(_constrain_statement) => todo!("Format constrain statement"),
            StatementKind::Expression(expression) => {
                chunks.text(self.chunk(|formatter| {
                    formatter.write_indentation();
                }));
                self.format_expression(expression, &mut chunks);
            }
            StatementKind::Assign(_assign_statement) => todo!("Format assign statement"),
            StatementKind::For(_for_loop_statement) => todo!("Format for loop statement"),
            StatementKind::Break => todo!("Format break statement"),
            StatementKind::Continue => todo!("Format continue statement"),
            StatementKind::Comptime(_statement) => todo!("Format comptime statement"),
            StatementKind::Semi(expression) => {
                chunks.text(self.chunk(|formatter| {
                    formatter.write_indentation();
                }));

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
